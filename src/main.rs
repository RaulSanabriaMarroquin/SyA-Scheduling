use std::collections::VecDeque; // la cola local por estación
use std::io::{self, Write};             // stdout + flush
use std::sync::{mpsc, Arc, Mutex};      // mpsc = multi-producer, single-consumer (canales tipados)
use std::thread;            // para crear hilos (std::thread::spawn)
use std::time::{Duration, Instant}; // Duration para sleep, Instant para medir tiempos
    // para simular “trabajo” con thread::sleep


// crossbeam para levantar hilos en scope
use crossbeam::thread as cb_thread;

// Struct es un TIPO con campos, se pasa por VALOR y cuando lo enviamos por un canal, transferimos la PROPIEDAD al siguiente hilo.
// Product es el producto que vamos a enviar por el canal.
#[derive(Debug, Clone)]
struct Product {
    id: u32,
    remaining_ms: Vec<u64>,      // trabajo restante por estación (para RR)
    arrival_ms: u128,            // tiempo de llegada a la línea (ms desde start)
    enter_ms: Vec<Option<u128>>, // tiempo de entrada por estación
    exit_ms:  Vec<Option<u128>>, // tiempo de salida por estación
}


// Esta es la estación de trabajo, que recibe productos por un canal y los envía a otro canal.
//name es el nombre de la estación y service_ms es el tiempo que tarda en procesar cada producto.

#[derive(Clone, Copy)]
struct StationCfg {
    id: usize,          // índice de estación (0,1,2) para indexar remaining_ms
    name: &'static str, // nombre para logs
    service_ms: u64,    // tiempo total de servicio requerido en esta estación
}

#[derive(Clone, Copy)]
enum Policy {
    Fcfs,
    RoundRobin { quantum_ms: u64 },
}

fn policy_name(p: Policy) -> &'static str {
    match p {
        Policy::Fcfs => "FCFS",
        Policy::RoundRobin { .. } => "RoundRobin",
    }
}

// Logger sincronizado para que no se mezclen prints entre hilos
fn logln(logger: &Arc<Mutex<io::Stdout>>, s: &str) {
    if let Ok(mut out) = logger.lock() {
        let _ = writeln!(&mut *out, "{s}");
        let _ = out.flush();
    }
}

fn now_ms(start: &Arc<Instant>) -> u128 {
    start.elapsed().as_millis()
}

// Inputs: cfg: configuración de la estación, rx: canal de entrada, tx_next: canal de salida
// recv() = BLOQUEA esperando un Product (hasta que llegue o hasta que se cierre el canal), retorna Result<Product, RecvError>
// Si recv() retorna Ok(p), entonces p es el Product recibido y si retorna Err(_), el canal está cerrado.
// send() = envía un Product al canal, retorna Result<(), SendError<Product>>
// Si send() retorna Err(_), el canal está cerrado y no se puede enviar más.
// La función corre en un bucle infinito hasta que el canal de entrada o salida se cierra.
// En resumen recv() bloquea esperando un Product, hace sleep para simular el trabajo y envia el Product al siguiente tramo con tx_next.send(p).
fn run_station(
    cfg: StationCfg, 
    policy: Policy,
    rx: mpsc::Receiver<Product>, 
    tx_next: mpsc::Sender<Product>,
    logger: Arc<Mutex<io::Stdout>>,
    start: Arc<Instant>,
) {
    let tid = thread::current().id();
    logln(
        &logger,
        &format!("⏳ Arranca hilo: {} (policy={}, tid={:?})", cfg.name, policy_name(policy), tid),
    );

    let mut ready: VecDeque<Product> = VecDeque::new(); // Cola local de listos
    let mut upstream_open = true; // al iniciar, asumimos upstream abierto

    loop {
        // 1) Si la cola está vacía, BLOQUEAMOS esperando al menos un item o cierre.
        if ready.is_empty() {
            logln(&logger, &format!("{}(tid={:?}) : cola vacía → esperando recv()", cfg.name, tid));
            match rx.recv() {
                // Si llega un item, lo agregamos a la cola
                Ok(p) => {
                    ready.push_back(p);
                    logln(&logger, &format!("{}(tid={:?}) : recibido → encola (cola=1)", cfg.name, tid));
                }
                // upstream se cerró y no hay nada en cola → terminar
                Err(_) => {
                    upstream_open = false;
                    logln(&logger, &format!("{}(tid={:?}) : upstream cerrado", cfg.name, tid));
                }
            }
            // si upstream se cerró y no entró nada, salimos
            if !upstream_open && ready.is_empty() {
                logln(&logger, &format!("{}(tid={:?}) : fin (sin items)", cfg.name, tid));
                break;
            }
        }
        // 2) Tomar el siguiente (FCFS = frente de la cola)
        let mut p = match ready.pop_front() {
            Some(x) => {
                logln(&logger, &format!(
                    "{}(tid={:?}) : toma Prod {:02} (cola restante={})",
                    cfg.name, tid, x.id, ready.len()
                ));
                x
            }
            None => {
                if !upstream_open {
                    logln(&logger, &format!("{}(tid={:?}) : fin (cola vacía tras cierre)", cfg.name, tid));
                    break;
                }
                continue;
            }
        };

        // Trabajo restante de este producto en ESTA estación
        let rem = p.remaining_ms[cfg.id];

        // Si por alguna razón ya estaba a 0 (defensivo), solo reenviar
        if rem == 0 {
            logln(&logger, &format!(
                "{}(tid={:?}) : Prod {:02} sin trabajo aquí → reenviar",
                cfg.name, tid, p.id
            ));
            if tx_next.send(p).is_err() {
                logln(&logger, &format!("{}(tid={:?}) : downstream cerrado → fin", cfg.name, tid));
                break;
            }
            continue;
        }

        // Marcar la PRIMERA entrada a esta estación
        if p.enter_ms[cfg.id].is_none() {
            p.enter_ms[cfg.id] = Some(now_ms(&start));
        }

        // 3) Aplicar la política para decidir cuánto "servicio" hacer en este turno
        let slice = match policy {
            Policy::Fcfs => rem, // procesar TODO de un tirón
            Policy::RoundRobin { quantum_ms } => rem.min(quantum_ms),
        };

        // Marca de entrada/slice
        logln(
            &logger,
            &format!(
                "→ {}(tid={:?}) : procesa Prod {:02} slice={}ms (restante antes={}ms)",
                cfg.name, tid, p.id, slice, rem
            ),
        );

        // Simular trabajo
        thread::sleep(Duration::from_millis(slice));

        // Actualizar restante
        p.remaining_ms[cfg.id] = rem - slice;

        if p.remaining_ms[cfg.id] == 0 {
            // Terminó esta estación
            logln(
                &logger,
                &format!(
                    "← {}(tid={:?}) : COMPLETÓ Prod {:02} → enviar a siguiente",
                    cfg.name, tid, p.id
                ),
            );
            // Enviar a la siguiente estación
            if tx_next.send(p).is_err() {
                logln(&logger, &format!("{}(tid={:?}) : downstream cerrado → fin", cfg.name, tid));
                break; // downstream cerrado
            } else {
                logln(&logger, &format!("{}(tid={:?}) : handoff → enviado", cfg.name, tid));
            }
        } else {
            // Aún le queda trabajo en esta estación → re-encolar (Round Robin)
            logln(
                &logger,
                &format!(
                    "↺ {}(tid={:?}) : re-encola Prod {:02} (restante después={}ms) (cola antes={})",
                    cfg.name, tid, p.id, p.remaining_ms[cfg.id], ready.len()
                ),
            );
            ready.push_back(p);
            logln(&logger, &format!("{}(tid={:?}) : cola después={}", cfg.name, tid, ready.len()));
        }

        // 4) Drenar llegadas nuevas sin bloquear
        let mut drained = 0usize;
        while let Ok(p_new) = rx.try_recv() {
            ready.push_back(p_new);
            drained += 1;
        }
        if drained > 0 {
            logln(
                &logger,
                &format!(
                    "{}(tid={:?}) : absorbió {} llegada(s) (cola={})",
                    cfg.name, tid, drained, ready.len()
                ),
            );
        }

        // 5) Si upstream cerró y ya no queda nada, salimos
        if !upstream_open && ready.is_empty() {
            logln(&logger, &format!("{}(tid={:?}) : fin (sin items)", cfg.name, tid));
            break;
        }
    }

    logln(&logger, &format!(" Termina hilo: {} (tid={:?})", cfg.name, tid));
}


// Tenemos 3 canales por cada estación (entrada y salida), y un canal para el sink final.
// Lanzamos un hilo por estaccion por cada estacion, con thread::spawn(move || { ... }):.
// move = transfiere la propiedad de las variables capturadas (rx,tx,cfg) al hilo.
// Esto es importante porque en Rust, las variables no pueden ser compartidas entre hilos sin sincronización explícita,
// y podria causar que un hilo externo siga usando recursos que pertenecen a un hilo interno.

fn main() {
    // Definimos las 3 estaciones y sus tiempos de servicio.
    let stations = [
        StationCfg { id: 0, name: "Corte",      service_ms: 800  },
        StationCfg { id: 1, name: "Ensamblaje", service_ms: 1200 },
        StationCfg { id: 2, name: "Empaque",    service_ms: 600  },
    ];
    // Definimos las políticas de cada estación
    let policies = [
        Policy::Fcfs,
        Policy::RoundRobin { quantum_ms: 400 },
        Policy::Fcfs,
    ];
    // Logger sincronizado + reloj base de la simulación
    let logger = Arc::new(Mutex::new(io::stdout()));
    let start  = Arc::new(Instant::now());

    // Creamos los canales 
    // mpsc::channel::<Product>() crea un canal tipado para Product, retornando (tx, rx)
    // Se puede clonar tx para tener múltiples productores; rx es único (single-consumer).
    // Cuando todos los tx se cierran, rx.recv() retorna Err, indicando que no hay más datos.
    
    // mpsc: pipes/colas entre estaciones
    let (tx_entry,  rx_entry) = mpsc::channel::<Product>(); // Entrada > Corte
    let (tx12,      rx12)     = mpsc::channel::<Product>(); // Corte > Ensamblaje
    let (tx23,      rx23)     = mpsc::channel::<Product>(); // Ensamblaje > Empaque
    let (tx_sink,   rx_sink)  = mpsc::channel::<Product>(); // Empaque > Sink

    // Para resumen final
    let total_service: u64 = stations.iter().map(|s| s.service_ms).sum();
    let mut final_order: Vec<u32> = Vec::new();
    let mut sum_tat: u128 = 0;   // suma Turnaround
    let mut sum_wait: u128 = 0;  // suma Espera

    // N productos + inter-arrival opcional
    let n: u32 = 6;
    let interarrival_ms = 150; // pon 150 si quieres ver solapamiento
 
  // ===== crossbeam::thread::scope: levantamos TODAS las tareas concurrentes =====
    cb_thread::scope(|s| {
        // Lanzar estaciones
        {
            let log = Arc::clone(&logger);
            let t0  = Arc::clone(&start);
            let rx = rx_entry;
            let tx = tx12.clone();
            s.spawn(move |_| run_station(stations[0], policies[0], rx, tx, log, t0));
        }
        {
            let log = Arc::clone(&logger);
            let t0  = Arc::clone(&start);
            let rx = rx12;
            let tx = tx23.clone();
            s.spawn(move |_| run_station(stations[1], policies[1], rx, tx, log, t0));
        }
        {
            let log = Arc::clone(&logger);
            let t0  = Arc::clone(&start);
            let rx = rx23;
            let tx = tx_sink.clone();
            s.spawn(move |_| run_station(stations[2], policies[2], rx, tx, log, t0));
        }

        // Generador
        {
            let tx = tx_entry.clone();
            let log = Arc::clone(&logger);
            let t0  = Arc::clone(&start);
            s.spawn(move |_| {
                for i in 1..=n {
                    // remaining inicial = servicio por estación
                    let remaining_ms: Vec<u64> = stations.iter().map(|s| s.service_ms).collect();
                    let arrival = now_ms(&t0);
                    let p = Product {
                        id: i,
                        remaining_ms,
                        arrival_ms: arrival,
                        enter_ms: vec![None; stations.len()],
                        exit_ms:  vec![None; stations.len()],
                    };
                    logln(&log, &format!("ENTRADA(gen) : Prod {:02} (arrival={}ms)", p.id, arrival));
                    tx.send(p).expect("No se pudo enviar a la entrada");
                    if interarrival_ms > 0 {
                        thread::sleep(Duration::from_millis(interarrival_ms));
                    }
                }
                drop(tx); // cierre upstream
            });
        }

        // Sink: calcula métricas y resumen
        {
            let log = Arc::clone(&logger);
            let t0  = Arc::clone(&start);
            s.spawn(move |_| {
                let mut local_order: Vec<u32> = Vec::with_capacity(n as usize);
                let mut local_sum_tat: u128 = 0;
                let mut local_sum_wait: u128 = 0;

                while let Ok(p) = rx_sink.recv() {
                    // fin al llegar al sink
                    let finish = now_ms(&t0);
                    let tat = finish - p.arrival_ms;                // turnaround
                    let wait = tat - (total_service as u128);       // espera total

                    // (Opcional) log por producto con tiempos por estación:
                    logln(&log, &format!(
                        "SINK: Prod {:02} FIN={}ms | TAT={}ms | WAIT={}ms | Corte({:?}→{:?}) Ensam({:?}→{:?}) Empa({:?}→{:?})",
                        p.id, finish, tat, wait,
                        p.enter_ms[0], p.exit_ms[0],
                        p.enter_ms[1], p.exit_ms[1],
                        p.enter_ms[2], p.exit_ms[2]
                    ));

                    local_order.push(p.id);
                    local_sum_tat  += tat;
                    local_sum_wait += wait;
                }

                // Resumen
                let avg_tat  = local_sum_tat as f64 / n as f64;
                let avg_wait = local_sum_wait as f64 / n as f64;
                logln(&log, "\n======== RESUMEN ========");
                logln(&log, &format!("Orden final de procesamiento: {:?}", local_order));
                logln(&log, &format!("Promedio Turnaround (ms): {:.2}", avg_tat));
                logln(&log, &format!("Promedio Espera     (ms): {:.2}", avg_wait));
                logln(&log, "=========================\n");
            });
        }
    }).expect("Error en scope de crossbeam");
}