#[macro_use]
extern crate clap;
mod functions;
use p2p_file_sharing_enum_commands::{
    CommandType, LsResponseType, ResponseType, StatusResponseType, PORT,
};
use clap::{Arg, App};
use std::net::{TcpStream, IpAddr, Ipv4Addr};
use std::io::Write;
use functions::{read_response, pretty_print};

fn main() {

    let matches = App::new("P2P File Sharing")
        .author(crate_authors!())
        .version(crate_version!())
        .usage("[share <filepath>] - Add to the list of files that are available for download from this client\n    \
                [scan] - Refresh the list of peers and files that are available for download\n    \
                [ls] - Show the list of files that this client can download\n    \
                [download <file_name> -o <save_path>] - Download the file with the specified name and save along the specified path\n    \
                [status] - Show a list of current downloads and a list of files that are available for download")
        .about("The application allows for decentralized file sharing")
        .arg(Arg::with_name("ARG1").required(true).help("Command name"))
        .arg(Arg::with_name("ARG2").help("FILE_PATH for the [share] command or FILE_NAME for the [download] command"))
        .arg(Arg::with_name("ARG3").help("SAVE PATH"))
        .arg(Arg::with_name("FLG1").help("Flag for saving with a forward string name").short("o"))
        .get_matches();

        if matches.is_present("ARG1"){

            if let Some(arg1_val) = matches.value_of("ARG1"){

                if arg1_val.to_lowercase() == "share" {
                    if let Some(arg2_val) = matches.value_of("ARG2") {

                    let command = CommandType::Share(arg2_val.to_string());
                    let to_daemon = serde_json::to_string(&command).unwrap();
                    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

                      match stream.write(to_daemon.as_bytes()) {

                          Ok(_) => {},
                          Err(e) => println!("Error: {} while stream transfers data", e)
                      }

                        match read_response(&mut stream) {
                            ResponseType::Error(err) => println!("{}", err),
                            _ => println!("Something wrong! Try again later.")
                        }
                    }
                        else {
                            println!("Error connection to a daemon!");
                        }
                    }
                }
                else if arg1_val.to_lowercase() == "scan"{

                    let command = CommandType::Scan;
                    let to_daemon = serde_json::to_string(&command).unwrap();
                    if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

                        match stream.write(to_daemon.as_bytes()) {

                            Ok(_) => {},
                            Err(e) => println!("Error: {} while stream transfers data", e)
                        }

                        match read_response(&mut stream) {
                            ResponseType::Error(err) => println!("{}", err),
                            _ => println!("Something wrong! Try again later")
                        }
                    }
                    else {
                        println!("Error connection to a daemon!");
                    }
                }
                else if arg1_val.to_lowercase() == "ls" {

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
                else if arg1_val.to_lowercase() == "download" {
                    if matches.is_present("ARG2"){
                        if let Some(arg2_val) = matches.value_of("ARG2"){

                            if let Ok(mut stream) = TcpStream::connect((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT)) {

                                if matches.is_present("FLG1") {
                                    if matches.is_present("ARG3") {

                                        if let Some(arg3_val) = matches.value_of("ARG3") {
                                            let command = CommandType::Download(arg2_val.to_string(), arg3_val.to_string());
                                            let to_daemon = serde_json::to_string(&command).unwrap();

                                            match stream.write(to_daemon.as_bytes()) {

                                                Ok(_) => {},
                                                Err(e) => println!("Error: {} while stream transfers data", e)
                                            }
                                        }
                                    }
                                }
                                else {

                                    let command = CommandType::Download(arg2_val.to_string(), String::new());
                                    let to_daemon = serde_json::to_string(&command).unwrap();
                                    match stream.write(to_daemon.as_bytes()) {

                                        Ok(_) => {},
                                        Err(e) => println!("Error: {} while stream transfers data", e)
                                    }
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
                    }
                }
                else if arg1_val.to_lowercase() == "status" {

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
            }
        }
}
