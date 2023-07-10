use serialport::{self, SerialPort};
use std::net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr};
use std::str;
use std::env;
use std::io::Read;

use crate::coords::Coordinates;
use crate::sync::{north_sync, sun_sync};
use crate::args::Configuration;

mod sync;
mod coords;
mod args;

/*
 * TODO: Add commandline interface
 */
fn main() {
    // Serial port configuration; by default it will always use the first one
    let ports = serialport::available_ports().unwrap();
    println!("Serialports: {:?} ", ports);
    let mut serial = serialport::new(&ports[0].port_name, 9600).open().unwrap();

    // Argument parsing
    let config = Configuration::parse_args(env::args().collect());
    
    let mut coords = north_sync(None);
    (serial, coords) = match (config.mode, config.gps, config.coax_azim, config.dish_azim) {
        (args::Mode::Sun, Some(coords), _, dish_azim) => {
            match sun_sync(serial, coords.0, coords.1, dish_azim) {
                (serial, Some(coordinates)) => (serial, coordinates),
                (_, None) => {return}
            }
        },

        (args::Mode::Sun, None, _, _) => {
            // Provided -S option without coordinates; we return
            return
        }

        (args::Mode::Direct, _, coaxial_azimuth, _) => {
            (serial, north_sync(coaxial_azimuth))
        },

        (args::Mode::Undef, _, _, _) => {
            // Failed to parse arguments; we return
            return
        }
    };

    let port = match config.port {
        Some(port) => port,
        None => 42690
    };
    let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), port);
    let listener = TcpListener::bind(socket_address).unwrap();

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
    let mut buf: [u8; 1024] = [0u8; 1024];
    let mut last_azim = 0f64;
    'read: while let Ok(size) = client.read(&mut buf[..]) {
        if size < 18 {println!(" Read: {} bytes", size); buf = [0u8; 1024]; continue 'read}
        let command = str::from_utf8(&buf).unwrap();
        let mut subcommand = command.split(" ");

        print!("Command: {}", command);

        match subcommand.next() {
            Some(command) => {
                match command {
                    // Add other commands here
                    "\\set_pos" | "P" => {
                        // Send received position to serialport
                        // Turns the coordinates from rotctld to those accepted by the VQ2500
                        // ..which turns out to be more complicated than I thought

                        // Reads the two numbers from the subcommand; format being set_pos <az> <el>
                        let mut azels: Vec<f64> = subcommand.map(|number| number.chars().filter(|c| c.is_digit(10) || *c == '.')
                                    .collect::<String>()
                                    .parse::<f64>().unwrap()).collect();

                        // Santitize the received numbers; azimuth must be in the 1-360 range and elevation must be 5-70 range
                        if azels[0] > 360.0 {azels[0] = azels[0] % 360.0;}
                        else if azels[0] < 1.0 && azels[0] > 0.0 {azels[0] = 360.0;}
                        else if azels[0] < 0.0 {azels[0] = azels[0] + 360.0;}
                        
                        if (azels[0] - last_azim).abs() > 0.2 {
                            last_azim = azels[0];
                        } else {
                            continue 'read
                        }
                        
                        //println!("With elements: {}, {}", azels[0], azels[1]);
                        if azels[1] < 5.0 {println!("Commanded elevation below limits!"); azels[1] = 5.0;}
                        else if azels[1] > 70.0 {println!("Commanded elevation over limits!"); azels[1] = 70.0;}
                        
                        /*
                         * ANGLE TO HEADING FORMULAS for VQ2500 (h: Heading (unitless); a: Angle (deg)):
                         * Azimuthal headings: 
                         *      <210* : h = 724.6 + 15.6 * a
                         *      >210* : h = 792.0 + 16.20 * a
                         * 
                         * Elevation headings:
                         *      h = 390.1 + 17.44 * a
                         */

                        // Adjusts the azimuthals
                        azels[0] = coords.to_internal(azels[0]);
                        println!("Into internal elements: {}, {}", azels[0], azels[1]);
                        // Turn those angles into headings according to the above formulas
                        if azels[0] >= 210.0 {
                            azels[0] = 792.0 + 16.20 * azels[0]; 
                        } else {
                            azels[0] = 724.6 + 15.6 * azels[0];
                        }

                        azels[1] = 390.1 + 17.44 * azels[1];

                        //println!("Turned into heading elements: {}, {}", azels[0], azels[1]);
                        let azimuthal_command = format!("azacc {:.0} \r\n", azels[0]);
                        let elevation_command = format!("elacc {:.0} \r\n", azels[1].ceil());
                        
                        // For some reason it only works if we send one byte at the time
                        azimuthal_command.bytes().for_each(|c| {serial.clear(serialport::ClearBuffer::All);serial.write(std::slice::from_ref(&c)).unwrap();});
                        elevation_command.bytes().for_each(|c| {serial.clear(serialport::ClearBuffer::All);serial.write(std::slice::from_ref(&c)).unwrap();});

                        //print!("{}", azimuthal_command);
                        //print!("{}", elevation_command);

                        // TODO: Get Response and flush it (or not lol)
                        // Wait until both operations are done to attempt receiving the next one
                    }
                    _ => {}
                }
            }
            None => {}            
        }
        buf = [0u8; 1024];
    }
    (serial, coords)
}
