use crate::common::{extract_file_name, send_file, DownloadData, StringVecIp};
use p2p_file_sharing_enum_commands::PORT;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use threadpool::ThreadPool;

pub fn listen_to_other_demons_via_tcp(
    my_files_to_share_list: StringVecIp,
    pool: Arc<Mutex<ThreadPool>>,
) {
    let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), PORT + 1)).unwrap();
    thread::spawn(move || {
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

                    for (key, _val) in my_files_to_share_list.lock().unwrap().iter() {
                        if download_data.file_name == extract_file_name(key.clone()) {
                            send_file(
                                &pool,
                                key.clone(),
                                my_files_to_share_list.clone(),
                                stream,
                                download_data,
                            );
                            break;
                        }
                    }
                }
                Err(err) => {
                    eprint!("Occurs error: {}", err);
                    continue;
                }
            }
        }
    });
}
