#[cfg(test)]
mod fifo;
#[cfg(test)]
mod lru;
#[cfg(test)]
mod reloj;

mod estadisticas;
mod paginacion;
mod tlb;
mod simulador;
mod experimentos;

use simulador::{Algoritmo, Simulador};

fn demostracion() {
    println!("=== SIMULADOR INTEGRADO DE MEMORIA VIRTUAL ===");
    let paginas = [1_u64, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];

    for algoritmo in [Algoritmo::FIFO, Algoritmo::LRU, Algoritmo::Reloj] {
        println!("\n======= {algoritmo:?} (3 marcos) =======");
        let mut sim = Simulador::nuevo(algoritmo, 3, 4);
        for pagina in paginas {
            let virtual_ = pagina * paginacion::TAMANO_PAGINA;
            match sim.acceder(virtual_) {
                Ok(fisica) => println!("Virtual: {virtual_} -> Física: {fisica}"),
                Err(error) => eprintln!("Error: {error}"),
            }
        }
        println!("\n--- ESTADÍSTICAS ---");
        sim.mostrar_resultados();
    }
}

fn main() {
    let demo = std::env::args().skip(1).any(|arg| arg == "--demo");
    if demo {
        demostracion();
        return;
    }

    println!("=====================================");
    println!(" SIMULADOR DE MEMORIA VIRTUAL EN RUST ");
    println!("=====================================\n");
    println!("Iniciando experimentos...\n");

    if let Err(error) = experimentos::ejecutar() {
        eprintln!("Error al ejecutar experimentos: {error}");
        std::process::exit(1);
    }
}
