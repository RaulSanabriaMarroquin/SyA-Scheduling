use std::sync::mpsc;        // mpsc = multi-producer, single-consumer (canales tipados)
use std::thread;            // para crear hilos (std::thread::spawn)
use std::time::Duration;    // para simular “trabajo” con thread::sleep

// Struct es un TIPO con campos, se pasa por VALOR y cuando lo enviamos por un canal, transferimos la PROPIEDAD al siguiente hilo.
struct Product {
    id: u32,
}

struct StationCfg {
    name: &'static str,
    service_ms: u64,
}

fn run_station(cfg: StationCfg, rx: mpsc::Receiver<Product>, tx_next: mpsc::Sender<Product>) {
    loop {
        match rx.recv() {
            Ok(p) => {
                println!("→ {}: entra Prod {:02}", cfg.name, p.id);
                thread::sleep(Duration::from_millis(cfg.service_ms));
                println!("← {}: sale  Prod {:02}", cfg.name, p.id);

                if tx_next.send(p).is_err() {
                    break; // downstream cerrado
                }
            }
            Err(_) => break, // upstream cerrado
        }
    }
}

fn main() {
    let corte       = StationCfg { name: "Corte",      service_ms: 800 };
    let ensamblaje  = StationCfg { name: "Ensamblaje", service_ms: 1200 };
    let empaque     = StationCfg { name: "Empaque",    service_ms: 600 };

    let (tx_entry,  rx_entry) = mpsc::channel::<Product>();
    let (tx12,      rx12)     = mpsc::channel::<Product>();
    let (tx23,      rx23)     = mpsc::channel::<Product>();
    let (tx_sink,   rx_sink)  = mpsc::channel::<Product>();

    // Estación Corte
    {
        let rx = rx_entry;
        let tx = tx12.clone();
        thread::spawn(move || run_station(corte, rx, tx));
    }
    // Estación Ensamblaje
    {
        let rx = rx12;
        let tx = tx23.clone();
        thread::spawn(move || run_station(ensamblaje, rx, tx));
    }
    // Estación Empaque
    {
        let rx = rx23;
        let tx = tx_sink.clone();
        thread::spawn(move || run_station(empaque, rx, tx));
    }

    // Generador mínimo
    for i in 1..=5 {
        tx_entry.send(Product { id: i }).unwrap();
    }
    drop(tx_entry); // señal de "no hay más"

    // Sink mínimo
    for _ in 1..=5 {
        let p = rx_sink.recv().unwrap();
        println!("✓ TERMINADO: Prod {:02}", p.id);
    }
}
