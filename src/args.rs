pub enum Mode {
    Sun,
    Direct,
    Undef
}
pub struct Configuration {
    pub port: Option<u16>,
    pub gps: Option<(f64,f64)>,
    pub coax_azim: Option<f64>,
    pub dish_azim: Option<f64>,
    pub mode: Mode
}

impl Configuration {
    fn usage() -> String {
        "Usage: astralis -p <port> [-S <lat> <lon> | -D] [-h <coax input heading>] [-d <dish azimuth>]
        where: -S <lat> <lon>\tSolar Alignement
               -D\t\tDirect aligment (should be used with -h; default: coax entrance pointing north)
               -h <heading>\tHeading of the coax input
               -p <port>\tPort to use for network (default: 42690)
               -d <heading>\tHeading of the antenna (in internal coordinates). Only used with -S".to_string()


    }
    pub fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn parse_args(args: Vec<String>) -> Configuration {
        // Skip the program name (1st argument)
        let mut config = Configuration {port: None, gps: None, coax_azim: None, dish_azim: None, mode: Mode::Undef};
        let (mut port_flag, mut sync_flag, mut heading_flag, mut dish_flag) = (false, false, false, false);
        for index in 1..args.len() {
            match &args[index][..] {
                "-p" | "--port" => {
                    if port_flag {continue;}
                    if let Some(port) = args.get(index+1) {
                        if let Ok(p)= port.parse::<u16>() {
                            config.port = Some(p);
                        } else {
                            println!("Unable to parse port number");
                            print!("{}", Configuration::usage());
                        }
                    } else {
                        println!("Unable to parse -p option");
                        print!("{}", Configuration::usage());
                    }
                    port_flag = true;
                }
                // Sun sync requires coordinates of the location
                "-S" | "--sun" => {
                    if sync_flag {continue;}
                    let strlat = args.get(index+1);
                    let strlong = args.get(index+2);

                    match (strlat, strlong) {
                        (Some(lt), Some(ln)) => {
                            let lat = lt.parse::<f64>();
                            let lon = ln.parse::<f64>();
                            match (lat, lon) {
                                (Ok(lat), Ok(lon)) => {
                                    if (-90.0 < lat && lat < 90.0) && (-180.0 < lon && lon < 180.0 ) {
                                        config.gps = Some((lat, lon));
                                        config.mode = Mode::Sun;
                                    }
                                }
                                _ => {
                                    println!("Unable to parse geographic coordinates!");
                                    print!("{}", Configuration::usage());
                                }
                            }
                        }
                        // Missing either or 
                        _ => {print!("{}", Configuration::usage());}
                    }
                    sync_flag = true;
                }

                "-D" | "--direct" => {
                    if sync_flag {continue;}
                    config.mode = Mode::Direct;
                    sync_flag = true;
                }

                "-h" | "--heading" => {
                    if heading_flag {continue;}
                    if let Some(heading) = args.get(index+1) {
                        if let Ok(h)= heading.parse::<f64>() {
                            config.coax_azim = Some(h);
                        } else {
                            println!("Unable to parse heading!");
                            print!("{}", Configuration::usage());
                        }
                    } else {
                        print!("{}", Configuration::usage());
                    }
                    heading_flag = true;
                }

                "-d" | "--dish" => {
                    if dish_flag {continue;}
                    if let Some(dish) = args.get(index+1) {
                        if let Ok(d)= dish.parse::<f64>() {
                            config.dish_azim = Some(d);
                        } else {
                            println!("Unable to parse dish heading");
                            print!("{}", Configuration::usage());
                        }
                    } else {
                        print!("{}", Configuration::usage());
                    }
                    dish_flag = true;
                }

                _ => {}
            }
        }
        todo!()
    }
}
