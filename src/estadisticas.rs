#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Estadisticas {
    pub referencias: usize,
    pub fallos: usize,
    pub aciertos: usize,
    pub reemplazos: usize,
}

impl Estadisticas {
    pub fn nuevas() -> Self {
        Self::default()
    }

    pub fn tasa_fallos(&self) -> f64 {
        if self.referencias == 0 {
            return 0.0;
        }
        self.fallos as f64 / self.referencias as f64 * 100.0
    }

    pub fn mostrar(&self) {
        println!("Referencias: {}", self.referencias);
        println!("Aciertos: {}", self.aciertos);
        println!("Fallos de página: {}", self.fallos);
        println!("Reemplazos: {}", self.reemplazos);
        println!("Tasa de fallos: {:.2}%", self.tasa_fallos());
    }

    pub fn calcular_eat(
        &self,
        fallos_tlb: usize,
        tiempo_tlb: f64,
        tiempo_ram: f64,
        tiempo_tabla: f64,
        tiempo_fallo: f64,
    ) -> f64 {
        if self.referencias == 0 {
            return 0.0;
        }
        assert!(fallos_tlb <= self.referencias);
        assert!(self.fallos <= self.referencias);

        let prob_fallo_tlb = fallos_tlb as f64 / self.referencias as f64;
        let prob_fallo_pagina = self.fallos as f64 / self.referencias as f64;

        tiempo_tlb + tiempo_ram
            + prob_fallo_tlb * tiempo_tabla
            + prob_fallo_pagina * tiempo_fallo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prueba_eat_sin_fallos_de_pagina() {
        let e = Estadisticas {
            referencias: 100,
            fallos: 0,
            aciertos: 100,
            reemplazos: 0,
        };
        assert_eq!(e.calcular_eat(20, 1.0, 100.0, 400.0, 8_000_000.0), 181.0);
    }

    #[test]
    fn prueba_estadisticas_vacias() {
        let e = Estadisticas::nuevas();
        assert_eq!(e.tasa_fallos(), 0.0);
        assert_eq!(e.calcular_eat(0, 1.0, 100.0, 400.0, 8_000_000.0), 0.0);
    }
}
