use spa::{solar_position, SolarPos, FloatOps};
use chrono::{TimeZone, Utc};

use serialport::{self, SerialPort};
use crate::coords::{self, Coordinates};
pub struct amd64;

macro_rules! implement {
    ($func:ident) => {
        fn $func(x: f64) -> f64 {x.$func()}
    };
    ($func:ident, $($fun:ident),+) => {
        implement!($func);
        implement!($($fun),+);
    }
}

impl FloatOps for amd64 {
    implement!(sin,cos,tan,asin,acos,atan,trunc);
    fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }
}

pub fn sun_sync(mut serial: Box<dyn SerialPort>, lat: f64, lon: f64) -> (Box<dyn SerialPort>, Coordinates) {
    let azimsol = solar_position::<amd64>(Utc::now(), lat, lon).unwrap().azimuth;
    // use serial to get the current heading and remember it as the azimuth given by the antenna
    serial.write(todo!()).unwrap();
    // read the return
    serial.read(todo!());
    (serial, Coordinates::new(todo!(), todo!()))
}

// The user guesses where the north is; points the coax input towards it.
// The program will therefor assume there is 0 offset. but because the antenna travels ccw instead of cw
// it's like if there were a 180* offset
pub fn north_sync() -> Coordinates {
    Coordinates::offset_new(180.0)
}