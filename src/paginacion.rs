use std::collections::HashMap;

pub const TAMANO_PAGINA: u64 = 4096;

#[derive(Debug)]
struct Entrada {
    marco: usize,
}

#[derive(Debug)]
enum Nodo {
    Tabla(HashMap<u16, Nodo>),
    Pagina(Entrada),
}

pub struct TablaPaginas {
    raiz: Nodo,
}

pub fn es_canonica(direccion: u64) -> bool {
    let bit_signo = (direccion >> 47) & 1;
    let bits_superiores = direccion >> 48;
    if bit_signo == 0 {
        bits_superiores == 0
    } else {
        bits_superiores == 0xFFFF
    }
}

fn indices(direccion: u64) -> [u16; 4] {
    [
        ((direccion >> 39) & 0x1FF) as u16,
        ((direccion >> 30) & 0x1FF) as u16,
        ((direccion >> 21) & 0x1FF) as u16,
        ((direccion >> 12) & 0x1FF) as u16,
    ]
}

impl TablaPaginas {
    pub fn nueva() -> Self {
        Self {
            raiz: Nodo::Tabla(HashMap::new()),
        }
    }

    pub fn mapear(&mut self, direccion: u64, marco: usize) -> Result<(), String> {
        if !es_canonica(direccion) {
            return Err("Dirección virtual no canónica".to_string());
        }
        let ruta = indices(direccion);
        let mut actual = &mut self.raiz;

        for indice in &ruta[..3] {
            actual = match actual {
                Nodo::Tabla(hijos) => hijos
                    .entry(*indice)
                    .or_insert_with(|| Nodo::Tabla(HashMap::new())),
                Nodo::Pagina(_) => return Err("Nivel de página inesperado".to_string()),
            };
        }

        match actual {
            Nodo::Tabla(hijos) => {
                hijos.insert(ruta[3], Nodo::Pagina(Entrada { marco }));
                Ok(())
            }
            Nodo::Pagina(_) => Err("Nivel de página inesperado".to_string()),
        }
    }

    pub fn traducir(&self, direccion: u64) -> Option<u64> {
        if !es_canonica(direccion) {
            return None;
        }

        let mut actual = &self.raiz;
        for indice in indices(direccion) {
            actual = match actual {
                Nodo::Tabla(hijos) => hijos.get(&indice)?,
                Nodo::Pagina(_) => return None,
            };
        }

        match actual {
            Nodo::Pagina(entrada) => {
                Some(entrada.marco as u64 * TAMANO_PAGINA + direccion % TAMANO_PAGINA)
            }
            Nodo::Tabla(_) => None,
        }
    }

    pub fn desmapear(&mut self, direccion: u64) -> bool {
        if !es_canonica(direccion) {
            return false;
        }
        let ruta = indices(direccion);
        let mut actual = &mut self.raiz;

        for indice in &ruta[..3] {
            actual = match actual {
                Nodo::Tabla(hijos) => match hijos.get_mut(indice) {
                    Some(hijo) => hijo,
                    None => return false,
                },
                Nodo::Pagina(_) => return false,
            };
        }

        match actual {
            Nodo::Tabla(hijos) => hijos.remove(&ruta[3]).is_some(),
            Nodo::Pagina(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prueba_traduccion() {
        let mut tabla = TablaPaginas::nueva();
        tabla.mapear(8192, 6).unwrap();
        assert_eq!(tabla.traducir(8196), Some(24580));
    }

    #[test]
    fn prueba_desalojo() {
        let mut tabla = TablaPaginas::nueva();
        tabla.mapear(8192, 6).unwrap();
        assert!(tabla.desmapear(8192));
        assert_eq!(tabla.traducir(8196), None);
    }

    #[test]
    fn prueba_direccion_no_canonica() {
        let mut tabla = TablaPaginas::nueva();
        let direccion = 0x0000_8000_0000_0000;
        assert!(tabla.mapear(direccion, 0).is_err());
        assert_eq!(tabla.traducir(direccion), None);
    }

    #[test]
    fn prueba_direccion_canonica_alta() {
        let mut tabla = TablaPaginas::nueva();
        let direccion = 0xFFFF_8000_0000_0004;
        tabla.mapear(direccion, 2).unwrap();
        assert_eq!(tabla.traducir(direccion), Some(8196));
    }
}
