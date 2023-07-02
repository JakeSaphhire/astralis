use serialport::{self, SerialPort};
use std::net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr, Incoming};
use std::str;
use std::io::Read;

use spa::{solar_position, SolarPos};
use chrono::{TimeZone, Utc};

use crate::coords::Coordinates;
use crate::sync::{north_sync, sun_sync};

mod sync;
mod coords;

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
    let mut buf = vec![0u8; 24];
    while let Ok(_size) = client.read(&mut buf[..]) {
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

                        // Reads the two numbers from the subcommand:
                        let mut azims: Vec<f64> = Vec::new();
                        for number in subcommand {
                            let num = number.chars().filter(|c| c.is_digit(10))
                                        .collect::<String>()
                                        .parse::<f64>().unwrap();
                            azims.push(coords.to_internal(num));
                        }

                        // Format the serial command string
                        let output = vec![0u8; 128];
                        serial.write(&buf[..]).unwrap();

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