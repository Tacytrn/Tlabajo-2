use std::collections::VecDeque;

use crate::estadisticas::Estadisticas;
use crate::paginacion::{es_canonica, TablaPaginas, TAMANO_PAGINA};
use crate::tlb::TLB;

#[derive(Clone, Copy, Debug)]
pub enum Algoritmo {
    FIFO,
    LRU,
    Reloj,
}

pub struct Simulador {
    algoritmo: Algoritmo,
    marcos: Vec<Option<u64>>,
    orden: VecDeque<u64>,
    bits: Vec<bool>,
    puntero: usize,
    tabla: TablaPaginas,
    tlb: TLB,
    estadisticas: Estadisticas,
}

impl Simulador {
    pub fn nuevo(algoritmo: Algoritmo, cantidad_marcos: usize, capacidad_tlb: usize) -> Self {
        assert!(cantidad_marcos > 0, "Se necesita al menos un marco físico");
        Self {
            algoritmo,
            marcos: vec![None; cantidad_marcos],
            orden: VecDeque::new(),
            bits: vec![false; cantidad_marcos],
            puntero: 0,
            tabla: TablaPaginas::nueva(),
            tlb: TLB::nueva(capacidad_tlb),
            estadisticas: Estadisticas::nuevas(),
        }
    }

    fn registrar_uso(&mut self, pagina: u64, marco: usize) {
        match self.algoritmo {
            Algoritmo::FIFO => {
      
            }
            Algoritmo::LRU => {
                if let Some(pos) = self.orden.iter().position(|&p| p == pagina) {
                    self.orden.remove(pos);
                }
                self.orden.push_back(pagina);
            }
            Algoritmo::Reloj => {
                self.bits[marco] = true;
            }
        }
    }

    fn elegir_marco(&mut self) -> usize {
        if let Some(pos) = self.marcos.iter().position(Option::is_none) {
            return pos;
        }

        match self.algoritmo {
            Algoritmo::FIFO | Algoritmo::LRU => {
                let victima = self.orden.pop_front().expect("Cola de reemplazo vacía");
                self.marcos.iter().position(|&p| p == Some(victima))
                    .expect("Página víctima ausente de RAM")
            }
            Algoritmo::Reloj => loop {
                let marco = self.puntero;
                self.puntero = (self.puntero + 1) % self.marcos.len();
                if !self.bits[marco] {
                    return marco;
                }
                self.bits[marco] = false;
            },
        }
    }


    pub fn acceder(&mut self, direccion: u64) -> Result<u64, String> {
        if !es_canonica(direccion) {
            return Err("Dirección virtual no canónica".to_string());
        }

        let pagina = direccion / TAMANO_PAGINA;
        let offset = direccion % TAMANO_PAGINA;
        self.estadisticas.referencias += 1;

        if let Some(marco) = self.tlb.buscar(pagina) {
            self.estadisticas.aciertos += 1;
            self.registrar_uso(pagina, marco);
            return Ok(marco as u64 * TAMANO_PAGINA + offset);
        }

        if let Some(fisica) = self.tabla.traducir(direccion) {
            let marco = (fisica / TAMANO_PAGINA) as usize;
            self.estadisticas.aciertos += 1;
            self.registrar_uso(pagina, marco);
            self.tlb.insertar(pagina, marco);
            return Ok(fisica);
        }

        self.estadisticas.fallos += 1;
        let marco = self.elegir_marco();

        if let Some(victima) = self.marcos[marco] {
            self.estadisticas.reemplazos += 1;
            self.tabla.desmapear(victima * TAMANO_PAGINA);
            self.tlb.invalidar(victima);
        }

        self.marcos[marco] = Some(pagina);
        self.bits[marco] = true;

        if matches!(self.algoritmo, Algoritmo::FIFO | Algoritmo::LRU) {
            self.orden.push_back(pagina);
        }

        self.tabla.mapear(direccion, marco)?;
        self.tlb.insertar(pagina, marco);
        Ok(marco as u64 * TAMANO_PAGINA + offset)
    }

    pub fn estadisticas(&self) -> &Estadisticas {
        &self.estadisticas
    }

    pub fn aciertos_tlb(&self) -> usize {
        self.tlb.aciertos()
    }

    pub fn fallos_tlb(&self) -> usize {
        self.tlb.fallos()
    }

    pub fn tasa_aciertos_tlb(&self) -> f64 {
        self.tlb.tasa_aciertos()
    }

    pub fn mostrar_resultados(&self) {
        self.estadisticas.mostrar();
        println!("Aciertos TLB: {}", self.aciertos_tlb());
        println!("Fallos TLB: {}", self.fallos_tlb());
        println!("Tasa de aciertos TLB: {:.2}%", self.tasa_aciertos_tlb());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fifo, lru, reloj};

    #[test]
    fn prueba_integracion() {
        for algoritmo in [Algoritmo::FIFO, Algoritmo::LRU, Algoritmo::Reloj] {
            let mut sim = Simulador::nuevo(algoritmo, 2, 2);
            assert_eq!(sim.acceder(8196).unwrap(), 4);
            assert_eq!(sim.acceder(8200).unwrap(), 8);
            assert_eq!(sim.estadisticas().fallos, 1);
            assert_eq!(sim.estadisticas().aciertos, 1);
        }
    }

    #[test]
    fn prueba_reemplazo_e_invalidacion() {
        for algoritmo in [Algoritmo::FIFO, Algoritmo::LRU, Algoritmo::Reloj] {
            let mut sim = Simulador::nuevo(algoritmo, 1, 2);
            sim.acceder(4096).unwrap();
            sim.acceder(8192).unwrap();
            assert_eq!(sim.estadisticas().reemplazos, 1);
            sim.acceder(4096).unwrap();
            assert_eq!(sim.estadisticas().fallos, 3);
        }
    }

    #[test]
    fn prueba_belady_y_comparacion_con_modulos() {
        let paginas = [1usize, 2, 3, 4, 1, 2, 5, 1, 2, 3, 4, 5];
        let direcciones: Vec<u64> = paginas.iter().map(|&p| p as u64 * 4096).collect();
        for (marcos, fifo_esperado, lru_esperado, reloj_esperado) in [
            (3, 9, 10, 9),
            (4, 10, 8, 10),
        ] {
            let esperados = [
                (Algoritmo::FIFO, fifo_esperado, fifo::simular(&paginas, marcos).fallos),
                (Algoritmo::LRU, lru_esperado, lru::simular(&paginas, marcos).fallos),
                (Algoritmo::Reloj, reloj_esperado, reloj::simular(&paginas, marcos).fallos),
            ];
            for (algoritmo, esperado, independiente) in esperados {
                let mut sim = Simulador::nuevo(algoritmo, marcos, 4);
                for &direccion in &direcciones {
                    sim.acceder(direccion).unwrap();
                }
                assert_eq!(sim.estadisticas().fallos, esperado);
                assert_eq!(sim.estadisticas().fallos, independiente);
                assert_eq!(sim.estadisticas().referencias, 12);
                assert_eq!(sim.aciertos_tlb() + sim.fallos_tlb(), 12);
            }
        }
    }

    #[test]
    fn prueba_fallo_tlb_sin_fallo_pagina() {
        let mut sim = Simulador::nuevo(Algoritmo::LRU, 3, 1);
        sim.acceder(4096).unwrap();
        sim.acceder(8192).unwrap();
        sim.acceder(4096).unwrap();
        assert_eq!(sim.estadisticas().fallos, 2);
        assert_eq!(sim.estadisticas().aciertos, 1);
        assert_eq!(sim.fallos_tlb(), 3);
        assert_eq!(sim.aciertos_tlb(), 0);
    }

    #[test]
    fn prueba_direccion_invalida_no_modifica_estado() {
        let mut sim = Simulador::nuevo(Algoritmo::FIFO, 3, 2);
        assert!(sim.acceder(0x0000_8000_0000_0000).is_err());
        assert_eq!(sim.estadisticas().referencias, 0);
        assert_eq!(sim.fallos_tlb(), 0);
    }
}
