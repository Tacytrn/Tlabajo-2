use std::collections::VecDeque;
use crate::estadisticas::Estadisticas;

pub fn simular(referencias: &[usize], cantidad_marcos: usize) -> Estadisticas {
    let mut memoria: VecDeque<usize> = VecDeque::new();
    let mut e = Estadisticas::nuevas();

    for &pagina in referencias {
        e.referencias += 1;
        if memoria.contains(&pagina) {
            e.aciertos += 1;
        } else {
            e.fallos += 1;
            if cantidad_marcos > 0 {
                if memoria.len() == cantidad_marcos {
                    memoria.pop_front();
                    e.reemplazos += 1;
                }
                memoria.push_back(pagina);
            }
        }
    }
    e
}
