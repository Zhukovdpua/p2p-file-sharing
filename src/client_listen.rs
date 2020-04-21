use crate::common::{
    push_to_hash_map, recv_file, IpVecString, RequestType, StringVecIp, MULTI_ADDR, UDP_PORT,
};

use p2p_file_sharing_enum_commands::{
    CommandType, LsResponseType, ResponseType, StatusResponseType, PORT,
};
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use threadpool::ThreadPool;

static mut AFTER_RESTART: bool = true;

fn process_client_command_buffer(
    buffer: &[u8],
    my_files_to_share_list: &StringVecIp,
    foreign_files_to_download_list: &IpVecString,
    pool: &Arc<Mutex<ThreadPool>>,
) -> Result<ResponseType, io::Error> {
    let command: CommandType = serde_json::from_str(&String::from_utf8_lossy(buffer))?;
    match command {
        CommandType::Share(file_name) => {
            push_to_hash_map(&my_files_to_share_list, file_name, vec![]);
            Ok(ResponseType::ShareScan)
        }
        CommandType::Scan => unsafe {
            let serialized_request_type = match AFTER_RESTART {
                true => {
                    AFTER_RESTART = false;
                    serde_json::to_string(&RequestType::ScanAfterRestart)?
                }
                false => serde_json::to_string(&RequestType::Scan)?,
            };
            UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))?
                .send_to(serialized_request_type.as_bytes(), (MULTI_ADDR, UDP_PORT))?;
            Ok(ResponseType::ShareScan)
        },
        CommandType::Ls => Ok(ResponseType::Ls(serde_json::to_string(
            &remove_tuple_to_ls_response(&foreign_files_to_download_list),
        )?)),
        CommandType::Download(file_name, save_path) => {
            match find_file(&foreign_files_to_download_list, file_name.clone()) {
                Some(ips) => {
                    let ips = filter_peers(&ips);
                    recv_file(
                        &pool,
                        ips,
                        save_path + &file_name,
                        &foreign_files_to_download_list,
                    );
                    Ok(ResponseType::Download(true))
                }
                None => Ok(ResponseType::Download(false)),
            }
        }
        CommandType::Status => {
            let containers_to_send: StatusResponseType = (
                select_sharing_files_to_send(&my_files_to_share_list),
                select_downloading_files_to_send(&foreign_files_to_download_list),
            );
            Ok(ResponseType::Status(serde_json::to_string(
                &containers_to_send,
            )?))
        }
    }
}

fn send_response_to_the_client(
    response: ResponseType,
    mut stream: &TcpStream,
) -> Result<(), io::Error> {
    stream.write_all(serde_json::to_string(&response)?.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn listen_to_client(
    my_files_to_share_list: StringVecIp,
    foreign_files_to_download_list: IpVecString,
    pool: Arc<Mutex<ThreadPool>>,
) {
    thread::spawn(move || {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), PORT)).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = vec![0; 4096];
                    let n = match stream.read(&mut buffer) {
                        Ok(n) => n,
                        Err(err) => {
                            eprint!("Occurs error: {}", err);
                            continue;
                        }
                    };
                    match process_client_command_buffer(
                        &buffer[..n],
                        &my_files_to_share_list,
                        &foreign_files_to_download_list,
                        &pool,
                    ) {
                        Ok(response) => {
                            if let Err(err) = send_response_to_the_client(response, &stream) {
                                eprint!("Occurs error: {}", err);
                            }
                        }
                        Err(err) => {
                            if let Err(err_inner) = send_response_to_the_client(
                                ResponseType::Error(format!("{:?}", err)),
                                &stream,
                            ) {
                                eprint!("Occurs error: {}", err_inner);
                            }
                            eprint!("Occurs error: {}", err);
                        }
                    }
                }
                Err(err) => {
                    eprint!("Occurs error: {}", err);
                }
            }
        }
    });
}

pub fn filter_peers(actual_peers: &[IpAddr]) -> Vec<IpAddr> {
    let mut result = Vec::new();
    let cpu_available = num_cpus::get() - 2;
    let mut i = 0;
    for peer in actual_peers {
        result.push(*peer);
        i += 1;
        if i >= cpu_available {
            break;
        }
    }
    result
}

pub fn remove_tuple_to_ls_response(foreign_files_to_download_list: &IpVecString) -> LsResponseType {
    let mut files_to_send = HashMap::new();
    for (key, val) in foreign_files_to_download_list.lock().unwrap().iter() {
        files_to_send.insert(*key, val.iter().map(|x| x.0.clone()).collect());
    }
    files_to_send
}

pub fn select_sharing_files_to_send(
    my_files_to_share_list: &StringVecIp,
) -> HashMap<String, Vec<IpAddr>> {
    let mut files_to_send = HashMap::new();
    for (key, val) in my_files_to_share_list.lock().unwrap().iter() {
        files_to_send.insert(
            key.clone(),
            val.iter().filter(|x| x.1).map(|x| x.0).collect(),
        );
    }
    files_to_send
}

pub fn select_downloading_files_to_send(
    foreign_files_to_download_list: &IpVecString,
) -> HashMap<String, Vec<IpAddr>> {
    let files_to_send = Arc::new(Mutex::new(HashMap::new()));
    for (key, val) in foreign_files_to_download_list.lock().unwrap().iter() {
        for (file_name, is_download) in val.iter() {
            if *is_download {
                push_to_hash_map(&files_to_send, file_name.clone(), vec![*key]);
            }
        }
    }
    let x = (&*files_to_send.lock().unwrap()).clone();
    x
}

pub fn find_file(
    foreign_files_to_download_list: &IpVecString,
    file_name: String,
) -> Option<Vec<IpAddr>> {
    let mut actual_peers = Vec::new();
    for (key, val) in foreign_files_to_download_list.lock().unwrap().iter() {
        if let Some(file_name) = val.iter().find(|x| x.0 == file_name) {
            if file_name.1 {
                return None;
            }
            actual_peers.push(*key);
        }
    }
    if !actual_peers.is_empty() {
        return Some(actual_peers);
    }
    None
}
