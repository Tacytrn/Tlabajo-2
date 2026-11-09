use crate::estadisticas::Estadisticas;

pub fn simular(referencias: &[usize], cantidad_marcos: usize) -> Estadisticas {
    let mut e = Estadisticas::nuevas();
    if cantidad_marcos == 0 {
        e.referencias = referencias.len();
        e.fallos = referencias.len();
        return e;
    }

    let mut memoria: Vec<Option<usize>> = vec![None; cantidad_marcos];
    let mut bits = vec![false; cantidad_marcos];
    let mut puntero = 0;

    for &pagina in referencias {
        e.referencias += 1;
        if let Some(pos) = memoria.iter().position(|&p| p == Some(pagina)) {
            e.aciertos += 1;
            bits[pos] = true;
        } else {
            e.fallos += 1;
            loop {
                if memoria[puntero].is_none() || !bits[puntero] {
                    if memoria[puntero].is_some() {
                        e.reemplazos += 1;
                    }
                    memoria[puntero] = Some(pagina);
                    bits[puntero] = true;
                    puntero = (puntero + 1) % cantidad_marcos;
                    break;
                }
                bits[puntero] = false;
                puntero = (puntero + 1) % cantidad_marcos;
            }
        }
    }
    e
}
