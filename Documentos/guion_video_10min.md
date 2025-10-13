# Guión para Video Explicativo - Tarea 2: Simulador de Línea de Ensamblaje

## Información General
- **Duración máxima**: 10 minutos
- **Formato**: Dos personas explicando (Persona A: Implementación, Persona B: Ejecución)
- **Audiencia**: Estudiantes de Sistemas Operativos
- **Objetivo**: Demostrar IPC, sincronización y algoritmos de scheduling

## 🎬 ESTRUCTURA DEL VIDEO (10 MINUTOS MÁXIMO)

### 1. INTRODUCCIÓN (1 minuto)
### 2. ARQUITECTURA DEL SISTEMA (2 minutos)
### 3. IMPLEMENTACIÓN TÉCNICA (2 minutos)
### 4. DEMOSTRACIÓN DE EJECUCIÓN (3 minutos)
### 5. ANÁLISIS DE RESULTADOS (1.5 minutos)
### 6. CONCLUSIÓN (0.5 minutos)

---

# 📝 GUION DETALLADO

## 1. INTRODUCCIÓN (1 minuto)

### Persona A:
> "Hola, soy [Nombre] y hoy vamos a explicar nuestra implementación de la Tarea 2: Simulador de Línea de Ensamblaje usando Rust."

### Persona B:
> "Y yo soy [Nombre], vamos a demostrar IPC, sincronización y algoritmos de scheduling en una línea de ensamblaje con tres estaciones."

### Persona A:
> "Implementamos FCFS, Round Robin y SJF usando Rust con hilos y canales mpsc."

**🎥 Acción en pantalla:**
- Mostrar título del proyecto
- Abrir README.md brevemente

## 2. ARQUITECTURA DEL SISTEMA (2 minutos)

### Persona A:
> "Tenemos tres estaciones: Corte (800ms), Ensamblaje (1200ms) y Empaque (600ms)."

**🎥 Acción en pantalla:**
- Mostrar líneas 282-286 del código

```rust
let stations = [
    StationCfg { id: 0, name: "Corte",      service_ms: 800  },
    StationCfg { id: 1, name: "Ensamblaje", service_ms: 1200 },
    StationCfg { id: 2, name: "Empaque",    service_ms: 600  },
];
```

### Persona B:
> "La comunicación se hace con canales mpsc: Entrada → Corte → Ensamblaje → Empaque → Sink."

**🎥 Acción en pantalla:**
- Mostrar líneas 305-309 del código

```rust
let (tx_entry,  rx_entry) = mpsc::channel::<Product>(); // Entrada > Corte
let (tx12,      rx12)     = mpsc::channel::<Product>(); // Corte > Ensamblaje
let (tx23,      rx23)     = mpsc::channel::<Product>(); // Ensamblaje > Empaque
let (tx_sink,   rx_sink)  = mpsc::channel::<Product>(); // Empaque > Sink
```

### Persona A:
> "Cada estación ejecuta en su propio hilo con políticas diferentes: Corte=FCFS, Ensamblaje=Round Robin, Empaque=SJF."

## 3. IMPLEMENTACIÓN TÉCNICA (2 minutos)

### Persona A:
> "Veamos los aspectos técnicos clave:"

### 3.1 Estructura de Datos

**🎥 Acción en pantalla:**
- Mostrar líneas 14-21 del código

```rust
struct Product {
    id: u32,
    remaining_ms: Vec<u64>,      // trabajo restante por estación
    arrival_ms: u128,            // tiempo de llegada
    enter_ms: Vec<Option<u128>>, // tiempo de entrada por estación
    exit_ms:  Vec<Option<u128>>, // tiempo de salida por estación
}
```

### Persona B:
> "Cada producto tiene ID único y registra tiempos para calcular métricas."

### 3.2 Algoritmos de Scheduling

### Persona A:
> "Implementamos tres algoritmos:"

**🎥 Acción en pantalla:**
- Mostrar líneas 288-292

```rust
let policies = [
    Policy::Fcfs,                    // Corte usa FCFS
    Policy::RoundRobin { quantum_ms: 400 }, // Ensamblaje usa RR
    Policy::Sjf,                     // Empaque usa SJF
];
```

### Persona B:
> "FCFS procesa todo de una vez, Round Robin usa quantum de 400ms, y SJF selecciona el trabajo más corto."

## 4. DEMOSTRACIÓN DE EJECUCIÓN (3 minutos)

### Persona B:
> "Ahora ejecutemos el simulador:"

**🎥 Acción en pantalla:**
- Ejecutar `cargo run`

### Persona A:
> "Vemos la inicialización de hilos:"

```
⏳ Arranca hilo: Corte (policy=FCFS, tid=ThreadId(2))
⏳ Arranca hilo: Ensamblaje (policy=RoundRobin, tid=ThreadId(3))
⏳ Arranca hilo: Empaque (policy=SJF, tid=ThreadId(4))
SINK: iniciando recepción de productos
```

### Persona B:
> "Observemos cómo cada estación espera productos inicialmente:"

```
Corte(tid=ThreadId(2)) : cola vacía → esperando recv()
Ensamblaje(tid=ThreadId(3)) : cola vacía → esperando recv()
Empaque(tid=ThreadId(4)) : cola vacía → esperando recv()
```

### Persona A:
> "Se generan 10 productos con llegadas escalonadas cada ~150ms:"

```
ENTRADA(gen) : Prod 01 (arrival=1ms)
ENTRADA(gen) : Prod 02 (arrival=151ms)
ENTRADA(gen) : Prod 03 (arrival=302ms)
ENTRADA(gen) : Prod 04 (arrival=453ms)
ENTRADA(gen) : Prod 05 (arrival=604ms)
ENTRADA(gen) : Prod 06 (arrival=754ms)
ENTRADA(gen) : Prod 07 (arrival=905ms)
ENTRADA(gen) : Prod 08 (arrival=1056ms)
ENTRADA(gen) : Prod 09 (arrival=1207ms)
ENTRADA(gen) : Prod 10 (arrival=1358ms)
Generador: terminó de enviar todos los productos
```

### Persona B:
> "Corte procesa con FCFS - toma productos completos de 800ms:"

```
Corte(tid=ThreadId(2)) : recibido → encola (cola=1)
Corte(tid=ThreadId(2)) : toma Prod 01 (cola restante=0)
→ Corte(tid=ThreadId(2)) : procesa Prod 01 slice=800ms (restante antes=800ms)
← Corte(tid=ThreadId(2)) : COMPLETÓ Prod 01 → enviar a siguiente
Corte(tid=ThreadId(2)) : handoff → enviado
Corte(tid=ThreadId(2)) : absorbió 5 llegada(s) (cola=5)
```

**Explicación:** Corte procesa completamente cada producto antes de pasar al siguiente. Mientras procesa, acumula productos en su cola.

### Persona A:
> "Ensamblaje usa Round Robin con quantum de 400ms:"

```
Ensamblaje(tid=ThreadId(3)) : recibido → encola (cola=1)
Ensamblaje(tid=ThreadId(3)) : toma Prod 01 (cola restante=0)
→ Ensamblaje(tid=ThreadId(3)) : procesa Prod 01 slice=400ms (restante antes=1200ms)
↺ Ensamblaje(tid=ThreadId(3)) : re-encola Prod 01 (restante después=800ms) (cola antes=0)
Ensamblaje(tid=ThreadId(3)) : cola después=1
```

**Explicación:** Round Robin procesa por quantum de 400ms, luego re-encola el producto con trabajo restante.

### Persona B:
> "Veamos cómo SJF en Empaque selecciona el trabajo más corto:"

```
Empaque(tid=ThreadId(4)) : recibido → encola (cola=1)
Empaque(tid=ThreadId(4)) : toma Prod 01 (cola restante=0)
→ Empaque(tid=ThreadId(4)) : procesa Prod 01 slice=600ms (restante antes=600ms)
← Empaque(tid=ThreadId(4)) : COMPLETÓ Prod 01 → enviar a siguiente
Empaque(tid=ThreadId(4)) : handoff → enviado
Empaque(tid=ThreadId(4)) : cola vacía → esperando recv()
```

**Explicación:** SJF siempre procesa completamente cada producto (600ms) sin interrupciones, pero cuando hay múltiples productos, selecciona el de menor tiempo restante.

### Persona A:
> "Observemos la sincronización entre estaciones:"

```
Corte(tid=ThreadId(2)) : absorbió 5 llegada(s) (cola=5)
Ensamblaje(tid=ThreadId(3)) : absorbió 1 llegada(s) (cola=2)
Empaque(tid=ThreadId(4)) : absorbió 1 llegada(s) (cola=1)
```

**Explicación:** Cada estación puede recibir múltiples productos mientras procesa, mostrando la concurrencia del sistema.

### Persona B:
> "Analicemos los patrones específicos del log:"

**Patrones de Round Robin:**
```
↺ Ensamblaje(tid=ThreadId(3)) : re-encola Prod 01 (restante después=800ms) (cola antes=0)
Ensamblaje(tid=ThreadId(3)) : cola después=1
```
**Significado:** El símbolo ↺ indica re-encolado. El producto se procesó por 400ms y queda 800ms restante.

**Patrones de SJF:**
```
Empaque(tid=ThreadId(4)) : toma Prod 01 (cola restante=0)
→ Empaque(tid=ThreadId(4)) : procesa Prod 01 slice=600ms (restante antes=600ms)
```
**Significado:** SJF procesa completamente cada producto sin interrupciones.

**Patrones de sincronización:**
```
Corte(tid=ThreadId(2)) : absorbió 5 llegada(s) (cola=5)
```
**Significado:** Mientras Corte procesaba un producto, llegaron 5 productos más que se acumularon en su cola.

### Persona A:
> "Veamos la progresión temporal del sistema:"

**Timeline de procesamiento:**
```
0ms    → Prod 01 llega a Corte
800ms  → Prod 01 termina Corte, va a Ensamblaje
1200ms → Prod 01 termina primer quantum en Ensamblaje (400ms)
1600ms → Prod 01 termina segundo quantum en Ensamblaje (400ms)
2000ms → Prod 01 termina tercer quantum en Ensamblaje (400ms)
2600ms → Prod 01 termina completamente en Empaque
```

**Explicación:** Round Robin requiere 3 quantums para completar 1200ms de trabajo.

### Persona B:
> "Observemos el efecto de la concurrencia en los tiempos:"

```
Prod 01: Corte(1→801) Ensam(802→2002) Empa(2002→2603)
Prod 02: Corte(802→1601) Ensam(2003→3605) Empa(3605→4206)
```

**Análisis:** 
- Prod 02 espera 1ms para entrar a Corte (801-802)
- Prod 02 espera 1ms para entrar a Ensamblaje (2002-2003)
- Prod 02 espera 0ms para entrar a Empaque (3605-3605)

**Conclusión:** El sistema mantiene flujo continuo con mínimas esperas entre estaciones.

## 5. ANÁLISIS DE RESULTADOS (1.5 minutos)

### Persona B:
> "Veamos las métricas detalladas por producto en el SINK:"

**🎥 Acción en pantalla:**
- Mostrar las líneas SINK del log

```
SINK: Prod 01 FIN=2606ms | TAT=2605ms | WAIT=5ms | Corte(Some(1)→None) Ensam(Some(802)→None) Empa(Some(2005)→None)
SINK: Prod 02 FIN=4207ms | TAT=4056ms | WAIT=1456ms | Corte(Some(802)→None) Ensam(Some(2005)→None) Empa(Some(3606)→None)
SINK: Prod 03 FIN=5409ms | TAT=5107ms | WAIT=2507ms | Corte(Some(1603)→None) Ensam(Some(2805)→None) Empa(Some(4808)→None)
SINK: Prod 04 FIN=7411ms | TAT=6958ms | WAIT=4358ms | Corte(Some(2404)→None) Ensam(Some(4007)→None) Empa(Some(6810)→None)
SINK: Prod 05 FIN=8612ms | TAT=8008ms | WAIT=5408ms | Corte(Some(3205)→None) Ensam(Some(4808)→None) Empa(Some(8011)→None)
```

**Explicación detallada:**
- **FIN**: Tiempo total desde inicio hasta finalización
- **TAT**: Turnaround Time = tiempo total de procesamiento del producto
- **WAIT**: Tiempo de espera = TAT - tiempo real de procesamiento
- **Corte/Ensam/Empa**: Tiempos de entrada y salida por estación

### Persona A:
> "Analicemos los patrones de tiempo de espera:"

```
Prod 01: WAIT=5ms    (primer producto, casi sin espera)
Prod 02: WAIT=1456ms (espera en colas de estaciones)
Prod 03: WAIT=2507ms (mayor congestión)
Prod 04: WAIT=4358ms (acumulación de productos)
Prod 05: WAIT=5408ms (máxima espera)
```

**Explicación:** Los tiempos de espera aumentan progresivamente debido a la acumulación de productos en las colas.

### Persona B:
> "Observemos cómo SJF afecta el orden de procesamiento en Empaque:"

```
Prod 01: Empa(Some(2005)→None) - procesado primero
Prod 02: Empa(Some(3606)→None) - procesado segundo  
Prod 03: Empa(Some(4808)→None) - procesado tercero
Prod 04: Empa(Some(6810)→None) - procesado cuarto
Prod 05: Empa(Some(8011)→None) - procesado quinto
```

**Explicación:** SJF mantiene el orden FCFS cuando todos los productos tienen el mismo tiempo de procesamiento (600ms).

### Persona A:
> "El resumen final muestra las estadísticas del sistema:"

```
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

### Persona B:
> "Todas las estaciones tienen 100% de utilización, indicando un sistema bien balanceado sin tiempo ocioso."

## 6. CONCLUSIÓN (0.5 minutos)

### Persona A:
> "Hemos implementado exitosamente un simulador que cumple todos los objetivos:"

### Persona B:
> "✅ IPC con canales mpsc
✅ Sincronización con Arc<Mutex<>> 
✅ Algoritmos FCFS, Round Robin y SJF
✅ Métricas de rendimiento completas"

### Ambos:
> "¡Gracias por ver nuestro video!"

**🎥 Acción en pantalla:**
- Mostrar información de contacto

---

# 📋 NOTAS TÉCNICAS PARA EL VIDEO

## Configuración de Pantalla
- **Resolución**: 1920x1080
- **División**: Izquierda (60%) código/terminal, Derecha (40%) cámaras
- **Herramientas**: OBS Studio, VS Code con tema claro

## Preparación Previa
1. **Compilar**: `cargo build`
2. **Limpiar terminal**: `cls`
3. **Archivos abiertos**: `src/main.rs`, `README.md`, terminal
4. **Scripts útiles**:
   ```bash
   cargo run | head -20
   cargo run > ejecucion_completa.log
   cargo run | grep "SINK:"
   ```

## Puntos Clave a Destacar
1. **IPC con mpsc**: Transferencia de propiedad entre hilos
2. **Sincronización**: Arc<Mutex<>> para recursos compartidos
3. **Algoritmos**: Diferencias entre FCFS, RR y SJF
4. **Métricas**: TAT, tiempo de espera, utilización
5. **Thread safety**: Ausencia de condiciones de carrera

## Timing del Video
- **Introducción**: 1 min
- **Arquitectura**: 2 min
- **Implementación**: 2 min
- **Ejecución**: 3 min
- **Análisis**: 1.5 min
- **Conclusión**: 0.5 min
- **Total**: 10 min

## Checklist Pre-Grabación
### Técnico:
- [ ] Proyecto compila sin errores
- [ ] Terminal configurado
- [ ] Archivos abiertos en VS Code
- [ ] OBS Studio configurado
- [ ] Audio y cámara funcionando

### Contenido:
- [ ] Guión revisado
- [ ] Código comentado listo
- [ ] Ejemplos de output listos
- [ ] Puntos clave identificados

### Presentación:
- [ ] Iluminación adecuada
- [ ] Audio claro
- [ ] Pantalla visible
- [ ] Presentadores preparados

---

# 🎯 OBJETIVOS DEL VIDEO

## Objetivos de Aprendizaje:
1. **Demostrar** implementación de IPC en Rust
2. **Explicar** técnicas de sincronización
3. **Comparar** algoritmos de scheduling
4. **Mostrar** análisis de métricas de rendimiento
5. **Ilustrar** buenas prácticas de programación concurrente

## Resultado Esperado:
Los espectadores deberían poder:
- Entender cómo implementar IPC con mpsc
- Diferenciar entre FCFS, Round Robin y SJF
- Interpretar métricas de rendimiento
- Aplicar técnicas de sincronización
- Implementar su propio simulador similar

---

## 📞 INFORMACIÓN DE CONTACTO

**Autores:**
- David Acuña López - rodolfoide69@estudiantec.cr
- Raúl Sanabria Marroquín - raulsanabria@estudiantec.cr

**Institución:**
- Escuela de Ingeniería en Computación
- Tecnológico de Costa Rica
- Cartago, Costa Rica

**Fecha:**
- Diciembre 2024
