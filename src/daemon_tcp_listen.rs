use crate::common::{extract_file_name, send_file, DownloadData, StringVecIp};
use p2p_file_sharing_enum_commands::PORT;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener};
use std::thread;

pub fn listen_to_other_demons_via_tcp(my_files_to_share_list: StringVecIp) {
    let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), PORT + 1)).unwrap();
    tokio::spawn(async move {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut download_data = vec![0; 4096];
                    let n = match stream.read(&mut download_data) {
                        Ok(n) => n,
                        Err(err) => {
                            eprint!("Occurs error: {}", err);
                            continue;
                        }
                    };

                    let download_data: DownloadData =
                        match serde_json::from_str(&String::from_utf8_lossy(&download_data[..n])) {
                            Ok(download_data) => download_data,
                            Err(err) => {
                                eprint!("Occurs error: {}", err);
                                continue;
                            }
                        };

                    let full_file_name = get_full_file_name(
                        &my_files_to_share_list,
                        download_data.file_name.clone(),
                    );
                    let mut my_files_to_share_list = my_files_to_share_list.clone();
                    thread::spawn(move || {
                        if let Err(err) = send_file(
                            full_file_name,
                            &mut my_files_to_share_list,
                            stream,
                            download_data,
                        ) {
                            eprint!("Occurs error: {}", err);
                        }
                    });
                }
                Err(err) => {
                    eprint!("Occurs error: {}", err);
                    continue;
                }
            }
        }
    });
}

fn get_full_file_name(my_files_to_share_list: &StringVecIp, file_name: String) -> String {
    let mut full_file_name = String::new();
    for (key, _val) in my_files_to_share_list.lock().unwrap().iter() {
        if file_name == extract_file_name(key.clone()) {
            full_file_name = key.clone();
            break;
        }
    }
    full_file_name
}
