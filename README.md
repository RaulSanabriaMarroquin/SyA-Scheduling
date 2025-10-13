# Simulador de Línea de Ensamblaje con Rust

## Descripción del Proyecto

Este proyecto implementa un simulador de línea de ensamblaje con tres estaciones conectadas en serie utilizando Rust. El objetivo es demostrar mecanismos de comunicación interprocesos IPC, técnicas de sincronización y algoritmos de planificación de procesos.

## Arquitectura del Sistema

### Estaciones de Trabajo
- **Corte**: Primera estación (800ms de servicio)
- **Ensamblaje**: Segunda estación (1200ms de servicio) 
- **Empaque**: Tercera estación (600ms de servicio)

### Algoritmos de Scheduling Implementados
- **FCFS (First-Come, First-Served)**: En la estación de Corte
- **Round Robin**: En la estación de Ensamblaje (quantum = 400ms)
- **SJF (Shortest Job First)**: En la estación de Empaque

## Características Técnicas

### Comunicación Interprocesos (IPC)
- Utiliza `std::sync::mpsc` para canales tipados entre estaciones
- Flujo unidireccional: Entrada → Corte → Ensamblaje → Empaque → Sink
- Transferencia de propiedad de datos entre hilos

### Sincronización
- `Arc<Mutex<...>>` para acceso thread-safe a recursos compartidos
- Logger sincronizado para evitar mezcla de output entre hilos
- Estadísticas globales compartidas entre todas las estaciones

### Manejo de Hilos
- `crossbeam::thread::scope` para manejo seguro de hilos
- Un hilo por estación de trabajo
- Hilos adicionales para generador de productos y sink de métricas

## Estructura del Código

### Estructuras Principales

```rust
struct Product {
    id: u32,
    remaining_ms: Vec<u64>,      // trabajo restante por estación
    arrival_ms: u128,            // tiempo de llegada
    enter_ms: Vec<Option<u128>>, // tiempo de entrada por estación
    exit_ms: Vec<Option<u128>>,  // tiempo de salida por estación
}

struct StationCfg {
    id: usize,
    name: &'static str,
    service_ms: u64,
}

struct GlobalStats {
    total_products_processed: Arc<Mutex<u32>>,
    total_processing_time: Arc<Mutex<u128>>,
    station_utilization: Arc<Mutex<Vec<f64>>>,
}
```

### Funciones Clave
- `run_station()`: Función principal de cada estación
- `policy_name()`: Convierte políticas a strings legibles
- `logln()`: Logger sincronizado
- `now_ms()`: Obtiene tiempo actual en milisegundos

## Instalación y Ejecución

### Prerrequisitos
- Rust
- Cargo

### Compilación
```bash
# Compilación en modo debug
cargo build

# Ejecutar directamente
cargo run
```

## Configuración

### Parámetros Modificables
- `n`: Número de productos a procesar (10)
- `interarrival_ms`: Tiempo entre llegadas (por defecto 150ms)
- `quantum_ms`: Quantum para Round Robin (por defecto 400ms)
- Tiempos de servicio por estación

### Ejemplo de Configuración
```rust
let stations = [
    StationCfg { id: 0, name: "Corte",      service_ms: 800  },
    StationCfg { id: 1, name: "Ensamblaje", service_ms: 1200 },
    StationCfg { id: 2, name: "Empaque",    service_ms: 600  },
];

let policies = [
    Policy::Fcfs,
    Policy::RoundRobin { quantum_ms: 400 },
    Policy::Sjf,
];
```

## Métricas y Análisis

### Métricas Registradas
- **Turnaround Time**: Tiempo total desde llegada hasta finalización
- **Waiting Time**: Tiempo de espera total
- **Utilización por Estación**: Porcentaje de tiempo activo
- **Orden de Procesamiento**: Secuencia final de productos

### Output del Simulador
```
Arranca hilo: Corte (policy=FCFS, tid=ThreadId(2))
Arranca hilo: Ensamblaje (policy=RoundRobin, tid=ThreadId(3))
Arranca hilo: Empaque (policy=SJF, tid=ThreadId(4))

ENTRADA(gen) : Prod 01 (arrival=0ms)
ENTRADA(gen) : Prod 02 (arrival=150ms)
...

======== RESUMEN ========
Orden final de procesamiento: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
Promedio Turnaround (ms): 2600.00
Promedio Espera     (ms): 0.00
Total productos procesados: 10
Utilización por estación:
  Corte: 100.0%
  Ensamblaje: 100.0%
  Empaque: 100.0%
=========================
```

## Análisis de Resultados

### Comportamiento de los Algoritmos
- **FCFS**: Procesa productos en orden de llegada sin interrupciones
- **Round Robin**: Alterna entre productos, mejorando equidad pero con overhead de conmutación
- **SJF**: Prioriza productos con menor tiempo restante, optimizando tiempo total

### Identificación de Cuellos de Botella
- La estación de Ensamblaje (1200ms) es típicamente el cuello de botella
- Las métricas de utilización ayudan a identificar estaciones subutilizadas

## Dependencias

```toml
[dependencies]
crossbeam = "0.8"
```

## Estructura del Proyecto

```
SyA-Scheduling/
├── src/
│   └── main.rs              # Código principal del simulador
├── Documentos/
│   ├── main.tex             # Documento LaTeX del proyecto
│   └── _PSO_TC2b__Comunicación_interprocesos.pdf
├── target/                 # Archivos de compilación
├── Cargo.toml              # Configuración del proyecto
├── Cargo.lock              # Lock file de dependencias
├── ejecucion.log           # Log de ejemplo
└── README.md               # Este archivo
```

## Trabajo Futuro

### Mejoras Propuestas
- Implementar LCFS (Last-Come, First-Served)
- Análisis de throughput bajo diferentes cargas
- Colas acotadas para simular memoria limitada
- Procesos en lugar de hilos para mayor realismo
- Interfaz gráfica para visualización en tiempo real

### Extensiones Técnicas
- Simulador de eventos discretos para mayor precisión
- Análisis estadístico avanzado
- Configuración dinámica de parámetros
- Métricas adicionales (throughput, latencia, etc.)

## Contribuciones

Este proyecto fue desarrollado como parte de un curso de Sistemas Operativos, demostrando conceptos fundamentales de:
- Comunicación interprocesos
- Sincronización de hilos
- Algoritmos de planificación
- Programación concurrente en Rust

## Realizado por

- **David Acuña López**: rodolfoide69@estudiantec.cr
- **Raúl Sanabria Marroquín**: raulsanabria@estudiantec.cr