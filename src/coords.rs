

pub struct Coordinates {
    // both in angles
    solaraz: f64,
    internalaz: f64,
    azoffset: f64,
}

impl Coordinates {
    pub fn new(solaz: f64, intaz: f64) -> Coordinates {
        Coordinates { solaraz: solaz, internalaz: intaz, azoffset: solaz - intaz}
    }

    pub fn offset_new(offset: f64) -> Coordinates {
        Coordinates {solaraz: -1.0, internalaz: -1.0, azoffset: offset }
    }

    pub fn to_internal(&self, tgt_azim: f64) -> f64 {
        let mut intaz = self.azoffset - tgt_azim;
        if intaz < 0.0 {
            intaz = 360.0 + intaz;
        } else if intaz > 360.0 {
            intaz = intaz - 360.0;
        } 
        intaz
    }
}