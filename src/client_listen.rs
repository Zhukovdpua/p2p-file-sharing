use crate::common::{
    push_to_hash_map, recv_file, IpVecString, RequestType, StringVecIp, MULTI_ADDR, UDP_PORT,
};

use p2p_file_sharing_enum_commands::{
    CommandType, LsResponseType, ResponseType, StatusResponseType, PORT,
};

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::prelude::*;

static mut AFTER_RESTART: bool = true;

fn process_client_command_buffer(
    buffer: &[u8],
    my_files_to_share_list: &StringVecIp,
    foreign_files_to_download_list: &IpVecString,
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
                    let ended_count = Arc::new(AtomicU64::new(0));

                    for i in 0..ips.len() {
                        let ip = ips[i];
                        let peer_count = ips.len() as u64;
                        let full_file_name = save_path.clone() + &file_name;
                        let mut foreign_files_to_download_list =
                            foreign_files_to_download_list.clone();
                        let ended_count = ended_count.clone();

                        tokio::spawn(async move {
                            if let Err(err) = recv_file(
                                ip,
                                full_file_name,
                                &mut foreign_files_to_download_list,
                                peer_count,
                                (i + 1) as u64,
                                &ended_count,
                            ) {
                                eprint!("Occurs error: {}", err);
                            }
                        });
                    }

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

pub fn listen_to_client(
    my_files_to_share_list: StringVecIp,
    foreign_files_to_download_list: IpVecString,
) {
    tokio::spawn(async move {
        let mut listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), PORT))
            .await
            .unwrap();
        loop {
            match listener.accept().await {
                Ok(connection) => {
                    let mut stream = connection.0;
                    let mut buffer = vec![0; 4096];
                    let n = match stream.read(&mut buffer).await {
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
                    ) {
                        Ok(response) => {
                            stream
                                .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                                .await;
                            stream.flush().await.unwrap();
                        }
                        Err(err) => {
                            stream
                                .write_all(
                                    serde_json::to_string(&ResponseType::Error(format!(
                                        "{:?}",
                                        err
                                    )))
                                    .unwrap()
                                    .as_bytes(),
                                )
                                .await;
                            stream.flush().await.unwrap();
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
