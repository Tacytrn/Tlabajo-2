use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct EntradaTLB {
    pagina_virtual: u64,
    marco_fisico: usize,
}

pub struct TLB {
    capacidad: usize,
    entradas: VecDeque<EntradaTLB>,
    aciertos: usize,
    fallos: usize,
}

impl TLB {
    pub fn nueva(capacidad: usize) -> Self {
        Self {
            capacidad,
            entradas: VecDeque::new(),
            aciertos: 0,
            fallos: 0,
        }
    }

    pub fn buscar(&mut self, pagina_virtual: u64) -> Option<usize> {
        if let Some(pos) = self.entradas.iter().position(|e| e.pagina_virtual == pagina_virtual) {
            self.aciertos += 1;
            let entrada = self.entradas.remove(pos).unwrap();
            let marco = entrada.marco_fisico;
            self.entradas.push_back(entrada);
            Some(marco)
        } else {
            self.fallos += 1;
            None
        }
    }

    pub fn insertar(&mut self, pagina_virtual: u64, marco_fisico: usize) {
        if self.capacidad == 0 {
            return;
        }
        self.invalidar(pagina_virtual);
        if self.entradas.len() == self.capacidad {
            self.entradas.pop_front();
        }
        self.entradas.push_back(EntradaTLB { pagina_virtual, marco_fisico });
    }

    pub fn invalidar(&mut self, pagina_virtual: u64) {
        if let Some(pos) = self.entradas.iter().position(|e| e.pagina_virtual == pagina_virtual) {
            self.entradas.remove(pos);
        }
    }

    pub fn aciertos(&self) -> usize {
        self.aciertos
    }

    pub fn fallos(&self) -> usize {
        self.fallos
    }

    pub fn tasa_aciertos(&self) -> f64 {
        let total = self.aciertos + self.fallos;
        if total == 0 {
            return 0.0;
        }
        self.aciertos as f64 / total as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prueba_acierto_y_fallo() {
        let mut tlb = TLB::nueva(2);
        assert_eq!(tlb.buscar(2), None);
        tlb.insertar(2, 6);
        assert_eq!(tlb.buscar(2), Some(6));
        assert_eq!(tlb.aciertos(), 1);
        assert_eq!(tlb.fallos(), 1);
        assert_eq!(tlb.tasa_aciertos(), 50.0);
    }

    #[test]
    fn prueba_reemplazo_lru() {
        let mut tlb = TLB::nueva(2);
        tlb.insertar(1, 0);
        tlb.insertar(2, 1);
        assert_eq!(tlb.buscar(1), Some(0));
        tlb.insertar(3, 2);
        assert_eq!(tlb.buscar(2), None);
        assert_eq!(tlb.buscar(1), Some(0));
    }

    #[test]
    fn prueba_invalidacion() {
        let mut tlb = TLB::nueva(2);
        tlb.insertar(2, 6);
        tlb.invalidar(2);
        assert_eq!(tlb.buscar(2), None);
    }

    #[test]
    fn prueba_capacidad_cero() {
        let mut tlb = TLB::nueva(0);
        tlb.insertar(2, 6);
        assert_eq!(tlb.buscar(2), None);
        assert_eq!(tlb.aciertos(), 0);
    }
}
