use spa::{solar_position, FloatOps};
use chrono::Utc;

use serialport::{self, SerialPort};
use crate::coords::Coordinates;
use std::str;
pub struct Amd64;

macro_rules! implement {
    ($func:ident) => {
        fn $func(x: f64) -> f64 {x.$func()}
    };
    ($func:ident, $($fun:ident),+) => {
        implement!($func);
        implement!($($fun),+);
    }
}

impl FloatOps for Amd64 {
    implement!(sin,cos,tan,asin,acos,atan,trunc);
    fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }
}

pub fn sun_sync(mut serial: Box<dyn SerialPort>, lat: f64, lon: f64, internal_azim: Option<f64>) -> (Box<dyn SerialPort>, Option<Coordinates>) {
    let azimsol = solar_position::<Amd64>(Utc::now(), lat, lon).unwrap().azimuth;
    /*
     * If the internal azimuth is set and provided by the user,
     * skip the azimuthal query to the antenna
     */
    if let Some(azim) = internal_azim {
        (serial, Some(Coordinates::new(azimsol, azim)))
    } else {
        serial.write(b"azacc\r\n").unwrap();
        let mut azacc_response : Vec<u8> = Vec::new();
        if let Ok(_size) = serial.read(&mut azacc_response[..]) {
            let response : Vec<f64> = str::from_utf8(&azacc_response[..]).unwrap().split('\n')
                .map(|part| part.chars().filter(|c| c.is_digit(10) || *c == '.')
                .collect::<String>().parse::<f64>().unwrap()).collect();

            (serial, Some(Coordinates::new(azimsol, response[1])))
        } else {
            // Failed to get response, *and* read from the antenna. Simply leave
            (serial, None)
        }
    }
}

// The user guesses where the north is; points the coax input towards it.
// The program will therefor assume there is 0 offset. but because the antenna travels ccw instead of cw
// it's like if there were a 180* offset
pub fn north_sync() -> Coordinates {
    Coordinates::offset_new(360.0)
}