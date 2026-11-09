# Trabajo 2 — Simulador de gestión de memoria y paginación en Rust

Proyecto Cargo `proyecto2` con paginación jerárquica de **4 niveles**, TLB de
capacidad configurable y tres algoritmos de reemplazo de páginas: **FIFO,
LRU y Reloj (segunda oportunidad)**.

## 1. Ejecutar en  (VS Code + Terminal)

Descomprime la carpeta y entra en ella:

```bash
cd ~/Downloads/proyecto2_completo
cargo test
cargo build --release
cargo run --release
```

> Ajusta el `cd` a donde guardaste la carpeta. Los comandos Cargo deben
> ejecutarse donde se encuentre `Cargo.toml`.

Al terminar se generan (o actualizan) los siguientes archivos:

- `resultados/algoritmos.csv`: 45 configuraciones (3 algoritmos × 5 marcos × 3 patrones).
- `resultados/tlb.csv`: 15 configuraciones (3 algoritmos × 5 capacidades TLB).
- `resultados/belady.csv`: 6 configuraciones (3 algoritmos × 2 marcos).
- `entradas/secuencial.csv`, `aleatorio.csv`, `localidad.csv`, `belady.csv`:
  **todas las referencias virtuales usadas**.

Para ver una ejecución detallada de los 12 accesos de Belady:

```bash
cargo run --release -- --demo
```

Para inspeccionar resultados:

```bash
cat resultados/belady.csv
head -n 12 resultados/algoritmos.csv
cat resultados/tlb.csv
```

## 2. Organización

| Archivo | Función |
| --- | --- |
| `src/main.rs` | Inicia los experimentos o la demostración (`--demo`). |
| `src/simulador.rs` | Integra memoria física, reemplazo, TLB y tabla de páginas. |
| `src/paginacion.rs` | Traducción con tabla jerárquica de cuatro niveles de 9 bits. |
| `src/tlb.rs` | Caché de traducciones con LRU interno e invalidación. |
| `src/estadisticas.rs` | Contadores, tasas y cálculo del EAT. |
| `src/experimentos.rs` | Patrones, mediciones repetidas, entradas y salidas CSV. |
| `src/fifo.rs`, `src/lru.rs`, `src/reloj.rs` | Implementaciones independientes usadas en las pruebas de consistencia. |


## 3. Diseño experimental

- Marcos físicos: **2, 4, 8, 16 y 32**.
- Algoritmos: **FIFO, LRU y Reloj**.
- Referencias por patrón: **10.000**.
- Patrones: `secuencial` (ciclo de 64 páginas), `aleatorio` (64 páginas)
  y `localidad` (80 % de accesos concentrados en 8 páginas; restante 20 %
  distribuido entre 64 páginas).
- Generador pseudoaleatorio LCG con **semilla fija 2026**.
- Experimento principal: TLB de 16 entradas.
- Experimento TLB: **32 marcos** y capacidades **4, 8, 16, 32 y 64**.
- Experimento Belady: cadena **1,2,3,4,1,2,5,1,2,3,4,5** con 3 y 4 marcos.
- Medición de tiempo: **10 ejecuciones** de cada configuración compilada con
  `--release`; promedio y desviación estándar poblacional en **microsegundos**.
  El tiempo incluye las referencias atendidas, no la creación inicial de `Simulador`.

Se cuentan fallos por cada 1000 referencias. Las mediciones en microsegundos
son **tiempos reales de ejecución del código Rust en el equipo donde se corre**.
Pueden variar entre máquinas y ejecuciones. No son tiempos de RAM/disco.

## 4. EAT: tiempo efectivo de acceso (modelo hipotético)

La función `Estadisticas::calcular_eat()` usa este modelo simplificado:

```text
EAT = T_TLB + T_RAM + P(miss_TLB) * T_tabla + P(fallo_pagina) * T_fallo
```

Costos elegidos para el modelo

- Consulta TLB: **1 ns**.
- Acceso RAM final: **100 ns**.
- Recorrido de la tabla de 4 niveles: **400 ns** adicionales si falla la TLB.
- Atención del fallo de página: **8 000 000 ns = 8 ms** adicionales.

Los campos `eat_ns` de `resultados/algoritmos.csv` y `resultados/tlb.csv`
registran el EAT estimado para cada configuración. Es un modelo de costos
que supone acceso final a RAM incluso después de atender el fallo; NO debe
interpretarse como el tiempo realmente observado con `Instant`.

## 5. Pruebas de funcionamiento

Ejecutar `cargo test`. Incluye pruebas de:

- Traducción y desmapeo de direcciones; direcciones no canónicas.
- TLB: acierto, fallo, capacidad LRU, invalidación y capacidad cero.
- Simulador integrado: traducciones, reemplazos, invalidación de TLB,
  misses de TLB sin fallos de página y comportamiento de Belady.
- Generación reproducible de referencias.
- Ejemplo numérico del EAT.

### Resultado esperado del experimento de Belady

| Algoritmo | Marcos 3 | Marcos 4 |
| --- | ---: | ---: |
| FIFO | 9 fallos | 10 fallos |
| LRU | 10 fallos | 8 fallos |
| Reloj | 9 fallos | 10 fallos |

## 6. Evidencia e informe

`documentacion/` incluye el enunciado original del trabajo.

`documentacion/resultados_reportados_en_chat/` contiene transcripciones
**de resultados que ya compartiste desde tu Mac**. Las mediciones de
`algoritmos` están incompletas en la conversación; NO se inventaron las filas
faltantes ni se presentan como salidas comprobadas de este contenedor.

Los CSV completos de `resultados/` se regeneran  con
`cargo run --release`; esos son los que debes usar para las gráficas y
el informe definitivo.

