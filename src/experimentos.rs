use std::fs::{self, File};
use std::hint::black_box;
use std::io::{self, BufWriter, Write};
use std::time::Instant;

use crate::paginacion::TAMANO_PAGINA;
use crate::simulador::{Algoritmo, Simulador};

const REPETICIONES: usize = 10;
const REFERENCIAS_POR_PATRON: usize = 10_000;
const SEMILLA: u64 = 2026;

const COSTO_TLB_NS: f64 = 1.0;
const COSTO_RAM_NS: f64 = 100.0;
const COSTO_TABLA_NS: f64 = 400.0;
const COSTO_FALLO_NS: f64 = 8_000_000.0;

#[derive(Debug)]
struct Medicion {
    referencias: usize,
    fallos: usize,
    aciertos: usize,
    reemplazos: usize,
    tlb_aciertos: usize,
    tlb_fallos: usize,
    tasa_tlb: f64,
    eat_ns: f64,
    promedio_us: f64,
    desviacion_us: f64,
}

fn siguiente_aleatorio(semilla: &mut u64) -> u64 {
    *semilla = semilla
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *semilla >> 32
}

fn generar_referencias(patron: &str, cantidad: usize) -> Vec<u64> {
    let mut datos = Vec::with_capacity(cantidad);
    let mut semilla = SEMILLA;

    for i in 0..cantidad {
        let pagina = match patron {
            "secuencial" => (i % 64) as u64,
            "aleatorio" => siguiente_aleatorio(&mut semilla) % 64,
            "localidad" => {
                let decision = siguiente_aleatorio(&mut semilla) % 100;
                if decision < 80 {
                    siguiente_aleatorio(&mut semilla) % 8
                } else {
                    siguiente_aleatorio(&mut semilla) % 64
                }
            }
            _ => panic!("Patrón desconocido: {patron}"),
        };
        datos.push(pagina * TAMANO_PAGINA);
    }
    datos
}

fn guardar_referencias(nombre: &str, direcciones: &[u64]) -> io::Result<()> {
    let ruta = format!("entradas/{nombre}.csv");
    let archivo = File::create(ruta)?;
    let mut csv = BufWriter::new(archivo);
    writeln!(csv, "indice,pagina_virtual,direccion_virtual")?;
    for (indice, &direccion) in direcciones.iter().enumerate() {
        writeln!(csv, "{indice},{},{}", direccion / TAMANO_PAGINA, direccion)?;
    }
    csv.flush()
}

fn medir(algoritmo: Algoritmo, marcos: usize, capacidad_tlb: usize, direcciones: &[u64]) -> Medicion {
    let mut tiempos = Vec::with_capacity(REPETICIONES);
    let mut resultado = None;

    for _ in 0..REPETICIONES {
        let mut sim = Simulador::nuevo(algoritmo, marcos, capacidad_tlb);
        let inicio = Instant::now();

        for &direccion in direcciones {
            let fisica = sim.acceder(black_box(direccion))
                .expect("Las referencias generadas son canónicas");
            black_box(fisica);
        }

        tiempos.push(inicio.elapsed().as_secs_f64() * 1_000_000.0);
        let e = sim.estadisticas();
        let conteos = (
            e.referencias,
            e.fallos,
            e.aciertos,
            e.reemplazos,
            sim.aciertos_tlb(),
            sim.fallos_tlb(),
        );
        if let Some(anterior) = resultado {
            assert_eq!(anterior, conteos, "Las repeticiones deben ser deterministas");
        }
        resultado = Some(conteos);
    }

    let promedio = tiempos.iter().sum::<f64>() / REPETICIONES as f64;
    let varianza = tiempos.iter().map(|t| (t - promedio).powi(2)).sum::<f64>()
        / REPETICIONES as f64;

    let (referencias, fallos, aciertos, reemplazos, tlb_aciertos, tlb_fallos) =
        resultado.expect("Debe existir al menos una repetición");

    let tasa_tlb = if referencias == 0 { 0.0 } else {
        100.0 * tlb_aciertos as f64 / referencias as f64
    };

    let est = crate::estadisticas::Estadisticas {
        referencias, fallos, aciertos, reemplazos,
    };
    let eat_ns = est.calcular_eat(
        tlb_fallos, COSTO_TLB_NS, COSTO_RAM_NS,
        COSTO_TABLA_NS, COSTO_FALLO_NS,
    );

    Medicion {
        referencias, fallos, aciertos, reemplazos,
        tlb_aciertos, tlb_fallos, tasa_tlb, eat_ns,
        promedio_us: promedio, desviacion_us: varianza.sqrt(),
    }
}

pub fn ejecutar() -> io::Result<()> {
    fs::create_dir_all("resultados")?;
    fs::create_dir_all("entradas")?;

    let algoritmos = [
        ("FIFO", Algoritmo::FIFO),
        ("LRU", Algoritmo::LRU),
        ("Reloj", Algoritmo::Reloj),
    ];

    let mut csv = BufWriter::new(File::create("resultados/algoritmos.csv")?);
    writeln!(csv, "algoritmo,patron,marcos,referencias,fallos,aciertos,reemplazos,fallos_por_1000,promedio_us,desviacion_us,tlb_aciertos,tlb_fallos,tasa_tlb,eat_ns")?;

    for patron in ["secuencial", "aleatorio", "localidad"] {
        let datos = generar_referencias(patron, REFERENCIAS_POR_PATRON);
        guardar_referencias(patron, &datos)?;

        for marcos in [2, 4, 8, 16, 32] {
            for &(nombre, algoritmo) in &algoritmos {
                let r = medir(algoritmo, marcos, 16, &datos);
                let fallos_por_1000 = r.fallos as f64 * 1000.0 / r.referencias as f64;
                writeln!(csv, "{nombre},{patron},{marcos},{},{},{},{},{:.3},{:.3},{:.3},{},{},{:.3},{:.3}",
                    r.referencias, r.fallos, r.aciertos, r.reemplazos,
                    fallos_por_1000, r.promedio_us, r.desviacion_us,
                    r.tlb_aciertos, r.tlb_fallos, r.tasa_tlb, r.eat_ns)?;
            }
        }
    }
    csv.flush()?;
    println!("Generado: resultados/algoritmos.csv");

    let mut csv = BufWriter::new(File::create("resultados/tlb.csv")?);
    writeln!(csv, "algoritmo,capacidad_tlb,referencias,tlb_aciertos,tlb_fallos,tasa_aciertos,fallos_pagina,eat_ns")?;
    let datos = generar_referencias("localidad", REFERENCIAS_POR_PATRON);

    for capacidad in [4, 8, 16, 32, 64] {
        for &(nombre, algoritmo) in &algoritmos {
            let r = medir(algoritmo, 32, capacidad, &datos);
            writeln!(csv, "{nombre},{capacidad},{},{},{},{:.3},{},{:.3}",
                r.referencias, r.tlb_aciertos, r.tlb_fallos,
                r.tasa_tlb, r.fallos, r.eat_ns)?;
        }
    }
    csv.flush()?;
    println!("Generado: resultados/tlb.csv");

    let mut csv = BufWriter::new(File::create("resultados/belady.csv")?);
    writeln!(csv, "algoritmo,marcos,fallos,aciertos")?;
    let paginas = [1_u64, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
    let direcciones: Vec<u64> = paginas.iter().map(|&p| p * TAMANO_PAGINA).collect();
    guardar_referencias("belady", &direcciones)?;

    for marcos in [3, 4] {
        for &(nombre, algoritmo) in &algoritmos {
            let r = medir(algoritmo, marcos, 4, &direcciones);
            writeln!(csv, "{nombre},{marcos},{},{}", r.fallos, r.aciertos)?;
        }
    }
    csv.flush()?;
    println!("Generado: resultados/belady.csv");

    println!("\nExperimentos terminados correctamente.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prueba_generador_reproducible() {
        assert_eq!(generar_referencias("aleatorio", 100), generar_referencias("aleatorio", 100));
        assert_eq!(generar_referencias("localidad", 100), generar_referencias("localidad", 100));
    }

    #[test]
    fn prueba_patron_secuencial() {
        let refs = generar_referencias("secuencial", 65);
        assert_eq!(refs[0], 0);
        assert_eq!(refs[63], 63 * TAMANO_PAGINA);
        assert_eq!(refs[64], 0);
    }
}
