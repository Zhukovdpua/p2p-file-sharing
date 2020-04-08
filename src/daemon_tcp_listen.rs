use crate::common::{extract_file_name, send_file, StringVecIp};
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
                    let mut file_name = vec![0; 4096];
                    let n = match stream.read(&mut file_name) {
                        Ok(n) => n,
                        Err(err) => {
                            eprint!("Occurs error: {}", err);
                            continue;
                        }
                    };
                    let file_name: String = String::from_utf8_lossy(&file_name[..n]).into(); // посмотрерть на cow<string>
                    for (key, _val) in my_files_to_share_list.lock().unwrap().iter() {
                        if file_name == extract_file_name(key.clone()) {
                            send_file(&pool, key.clone(), my_files_to_share_list.clone(), stream);
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
