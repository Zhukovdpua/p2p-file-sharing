#[macro_use]
extern crate clap;
use commands::CommandType;
//use commands::ResponseType;
use clap::{Arg, App};
use std::net::{TcpStream, IpAddr};
use std::io::{Read, Write};
use std::vec;
use std::collections::HashMap;

fn main() {

    //should we use multiple for args ?
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
                    if let Ok(mut stream) = TcpStream::connect("localhost:8080") {

                      match stream.write(to_daemon.as_bytes()) {

                          Ok(_) => println!("Your [share] request has been sent"),
                          Err(e) => println!("Error: {} while stream transfers data", e)
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
                    if let Ok(mut stream) = TcpStream::connect("localhost:8080") {

                        match stream.write(to_daemon.as_bytes()) {

                            Ok(_) => println!("Your [scan] request has been sent"),
                            Err(e) => println!("Error: {} while stream transfers data", e)
                        }
                    }
                    else {
                        println!("Error connection to a daemon!");
                    }
                }
                else if arg1_val.to_lowercase() == "ls" {

                    let command = CommandType::Ls;
                    let to_daemon = serde_json::to_string(&command).unwrap();
                    if let Ok(mut stream) = TcpStream::connect("localhost:8080") {

                        match stream.write(to_daemon.as_bytes()) {
                            Ok(_) => println!("Your [ls] request has been sent"),
                            Err(e) => println!("Error: {} while stream transfers data", e)
                        }

                        let mut buf = vec![];
                        loop {
                            match stream.read_to_end(&mut buf) {
                                Ok(_) => break,
                                Err(e) => panic!("encountered IO error: {}", e),
                            };
                        };

                        let s = String::from_utf8_lossy(&buf);
                        let result: HashMap<IpAddr, Vec<String>> = serde_json::from_str(&s).unwrap();
                        println!("{:?}", result);
                    }
                    else {

                        println!("Error connection to the daemon!");
                    }
                }
                else if arg1_val.to_lowercase() == "download" {
                    if matches.is_present("ARG2"){
                        if matches.is_present("FLG1"){
                            if matches.is_present("ARG3"){

                                if let Some(arg2_val) = matches.value_of("ARG2"){

                                        if let Some(arg3_val) = matches.value_of("ARG3"){

                                                let command = CommandType::Download(arg2_val.to_string(), arg3_val.to_string());
                                                let to_daemon = serde_json::to_string(&command).unwrap();
                                                if let Ok(mut stream) = TcpStream::connect("localhost:8080") {

                                                    match stream.write(to_daemon.as_bytes()) {

                                                        Ok(_) => println!("Your [download] request has been sent"),
                                                        Err(e) => println!("Error: {} while stream transfers data", e)
                                                    }
                                            }
                                            else {
                                                println!("Error connection to a daemon!");
                                            }
                                        }
                                }
                            }
                        }
                    }
                }
                else if arg1_val.to_lowercase() == "status" {

                    let command = CommandType::Status;
                    let to_daemon = serde_json::to_string(&command).unwrap();
                    if let Ok(mut stream) = TcpStream::connect("localhost:8080") {

                        match stream.write(to_daemon.as_bytes()) {

                            Ok(_) => println!("Your share [status] has been sent"),
                            Err(e) => println!("Error: {} while stream transfers data", e)
                        }

                        let mut buf = vec![];
                        loop {
                            match stream.read_to_end(&mut buf) {
                                Ok(_) => break,
                                Err(e) => panic!("encountered IO error: {}", e),
                            };
                        };
                        let s = String::from_utf8_lossy(&buf);
                        let result : (HashMap<String, Vec<IpAddr>>, HashMap<String, Vec<IpAddr>>) = serde_json::from_str(&s).unwrap();
                        println!("{:?}", result);
                    }
                    else {
                        println!("Error connection to a daemon!");
                    }
                }
            }
        }
}
