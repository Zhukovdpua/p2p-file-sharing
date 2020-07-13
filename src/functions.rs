use std::net::{TcpStream, IpAddr};
use std::collections::HashMap;
use p2p_file_sharing_enum_commands::ResponseType;
use std::io::Read;

pub type Filename = String;
pub type Container = HashMap<Filename, Vec<IpAddr>>;

    pub fn read_response (connection: &mut TcpStream) -> ResponseType {

        let mut buf: Vec<u8> = vec![];
        let n = connection.read_to_end(&mut buf).unwrap();
        let command: ResponseType = serde_json::from_str(&String::from_utf8_lossy(&buf[..n])).unwrap();

        command
    }

    fn pretty_print_container(container: &Container) {
        for (filename, addresses) in container.into_iter() {
            println!("  The file '{filename}' are downloading peers:", filename = filename);

            for address in addresses {
                println!("      {address}", address = address);
            }
        }
    }

   pub fn pretty_print(containers: (&Container, &Container)) {
        println!("Files to download:");
        pretty_print_container(containers.0);
        println!("------------------------------");
        println!("Downloading files:");
        pretty_print_container(containers.1);
    }
