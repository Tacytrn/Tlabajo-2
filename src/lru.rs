use std::collections::VecDeque;
use crate::estadisticas::Estadisticas;

pub fn simular(referencias: &[usize], marcos: usize) -> Estadisticas {
    let mut memoria: VecDeque<usize> = VecDeque::new();
    let mut e = Estadisticas::nuevas();

    for &pagina in referencias {
        e.referencias += 1;
        if let Some(posicion) = memoria.iter().position(|&p| p == pagina) {
            e.aciertos += 1;
            memoria.remove(posicion);
            memoria.push_back(pagina);
        } else {
            e.fallos += 1;
            if marcos == 0 {
                continue;
            }
            if memoria.len() == marcos {
                memoria.pop_front();
                e.reemplazos += 1;
            }
            memoria.push_back(pagina);
        }
    }
    e
}
