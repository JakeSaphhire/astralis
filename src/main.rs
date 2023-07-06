use serialport::{self, SerialPort};
use std::net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr};
use std::str;
use std::io::Read;

use crate::coords::Coordinates;
use crate::sync::{north_sync, sun_sync};

mod sync;
mod coords;

/*
 * TODO: Add commandline interface
 * TODO: Add commandline argument parsing
 * TODO: Fix (or remove) northsync
 */
fn main() {

    let ports = serialport::available_ports().unwrap();
    println!("Serialports: {:?} ", ports);
    let mut serial = serialport::new(&ports[0].port_name, 9600).open().unwrap();

    let port = 42690;
    let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), port);
    let listener = TcpListener::bind(socket_address).unwrap();

    // Temporary
    let mut coords = north_sync();

    // Wait for a single connection and handle it
    println!("Awaiting Connection...");
    for client in listener.incoming() {
        match client {
            Ok(client) => {(serial, coords) = into_handle(client, serial, coords);},
            Err(e) => {println!("Connection error: {}", e);} 
        }

    }
}

fn into_handle(mut client: TcpStream, mut serial: Box<dyn SerialPort>, coords: Coordinates) -> (Box<dyn SerialPort>, Coordinates) {
    let mut buf = vec![0u8; 48];
    'read: while let Ok(_size) = client.read(&mut buf[..]) {
        let command = str::from_utf8(&buf).unwrap();
        let mut subcommand = command.split(" ");

        match subcommand.next() {
            Some(command) => {
                match command {
                    // Add other commands here
                    "\\set_pos" | "P" => {
                        // Send received position to serialport
                        // Turns the coordinates from rotctld to those accepted by the VQ2500
                        // ..which turns out to be more complicated than I thought

                        // Reads the two numbers from the subcommand; format being set_pos <az> <el>
                        let mut azels: Vec<f64> = subcommand.map(|number| number.chars().filter(|c| c.is_digit(10))
                                    .collect::<String>()
                                    .parse::<f64>().unwrap()).collect();

                        // Santitize the received numbers; azimuth must be in the 1-360 range and elevation must be 5-70 range
                        if azels[0] > 360.0 {azels[0] = azels[0] % 360.0;}
                        else if azels[0] < 1.0 && azels[0] > 0.0 {azels[0] = 360.0;}
                        else if azels[0] < 0.0 {azels[0] = azels[0] + 360.0;}
                        
                        if azels[1] < 5.0 {println!("Commanded elevation below limits!"); azels[1] = 5.0;}
                        else if azels[1] > 70.0 {println!("Commanded elevation over limits!"); azels[1] = 70.0;}

                        // Turn those azels into interal coords
                        azels.iter_mut().for_each(|angle| *angle = coords.to_internal(*angle));
                        
                        /*
                         * TODO: Control small movements with nudges instead of angles
                         * ANGLE TO HEADING FORMULAS for VQ2500 (h: Heading (unitless); a: Angle (deg)):
                         * Azimuthal headings: 
                         *      <210* : h = 724.6 +- 15.6 * a
                         *      >210* : h = 792.0 + 16.20 * a
                         * 
                         * Elevation headings:
                         *      h = 390.1 + 17.44 * a
                         */

                        // Turn those angles into headings according to the above formulas
                        if azels[0] >= 210.0 {
                            azels[0] = 792.0 + 16.20 * azels[0]; 
                        } else {
                            azels[0] = 724.6 + 15.6 * azels[0];
                        }

                        azels[1] = 390.1 + 17.44 * azels[1];

                        let azimuthal_command = format!("azacc {:.0} \r\n", azels[0]);
                        let elevation_command = format!("elacc {:.0} \r\n", azels[1]);
                        serial.write(azimuthal_command.as_bytes()).unwrap();
                        serial.write(elevation_command.as_bytes()).unwrap();

                        // TODO: Get Response and flush it (or not lol)
                        // Wait until both operations are done to attempt receiving the next one
                    }
                    _ => {}
                }
            }
            None => {}            
        }
    }
    (serial, coords)
}