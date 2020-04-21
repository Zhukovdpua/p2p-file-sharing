use p2p_file_sharing_enum_commands::{
    CommandType, LsResponseType, ResponseType, StatusResponseType, PORT,
};
use std::net::{TcpStream, IpAddr, Ipv4Addr};
use std::io::Write;
use super::functions::{read_response, pretty_print};

pub fn share(file_path: &str) {

    let command = CommandType::Share(file_path.to_string());
    let to_daemon = serde_json::to_string(&command).unwrap();
    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

        match stream.write(to_daemon.as_bytes()) {

            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data", e)
        }

        match read_response(&mut stream) {

            ResponseType::ShareScan => {},
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later.")
        }
    }
    else {
        println!("Error connection to a daemon!");
    }
}

pub fn scan() {

    let command = CommandType::Scan;
    let to_daemon = serde_json::to_string(&command).unwrap();
    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

        match stream.write(to_daemon.as_bytes()) {

            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data", e)
        }

        match read_response(&mut stream) {

            ResponseType::ShareScan => {},
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later")
        }
    }
    else {
        println!("Error connection to a daemon!");
    }
}

pub fn ls(){

    let command = CommandType::Ls;
    let to_daemon = serde_json::to_string(&command).unwrap();
    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

        match stream.write(to_daemon.as_bytes()) {
            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data", e)
        }

        match read_response(&mut stream) {
            ResponseType::Ls(str) => {
                let container: LsResponseType = serde_json::from_str(&str).unwrap();
                println!("{:?}", container);
            },
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later")
        }
    }
    else {

        println!("Error connection to the daemon!");
    }
}

pub fn  download_with_path(file_name: &str, file_path: &str) {

    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

        let command = CommandType::Download(file_name.to_string(), file_path.to_string());
        let to_daemon = serde_json::to_string(&command).unwrap();

        match stream.write(to_daemon.as_bytes()) {

            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data", e)
        }

        match read_response(&mut stream) {
            ResponseType::Download(answer) => {
                match answer {
                    true => println!("Download started!"),
                    false => println!("The file does not exist!")
                }
            },
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later")
        }
    }
    else {
            println!("Error connection to a daemon!");
    }
}

pub fn download_without_path(file_name: &str){

    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {
        let command = CommandType::Download(file_name.to_string(), String::new());
        let to_daemon = serde_json::to_string(&command).unwrap();

        match stream.write(to_daemon.as_bytes()) {
            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data", e)
        }

        match read_response(&mut stream) {
            ResponseType::Download(answer) => {
                match answer {
                    true => println!("Download started!"),
                    false => println!("The file does not exist!")
                }
            },
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later")
        }
    }
    else {
        println!("Error connection to a daemon!");
    }
}

pub fn status () {

    let command = CommandType::Status;
    let to_daemon = serde_json::to_string(&command).unwrap();
    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

        match stream.write(to_daemon.as_bytes()) {

            Ok(_) => {},
            Err(e) => println!("Error: {} while stream transfers data!", e)
        }

        match read_response(&mut stream) {
            ResponseType::Status(str) => {
                let container: StatusResponseType = serde_json::from_str(&str).unwrap();
                pretty_print((&container.0, &container.1));
            },
            ResponseType::Error(err) => println!("{}", err),
            _ => println!("Something wrong! Try again later")
        }
    }
    else {
        println!("Error connection to a daemon!");
    }
}
