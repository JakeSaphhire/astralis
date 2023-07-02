

pub struct Coordinates {
    // both in angles
    solaraz: f64,
    internalaz: f64,
    azoffset: f64,
}

impl Coordinates {
    pub fn new(solaz: f64, intaz: f64) -> Coordinates {
        Coordinates { solaraz: solaz, internalaz: intaz, azoffset: intaz - solaz}
    }

    pub fn offset_new(offset: f64) -> Coordinates {
        Coordinates {solaraz: -1.0, internalaz: -1.0, azoffset: offset }
    }

    pub fn to_internal(&self, azim: f64) -> f64 {
        let mut intaz = azim + self.azoffset;
        if intaz < 0.0 {
            intaz = intaz + 360.0;
        } else if intaz > 360.0 {
            intaz = intaz - 360.0;
        } 
        intaz
    }
}