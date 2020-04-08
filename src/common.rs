use p2p_file_sharing_enum_commands::PORT;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

pub const MULTI_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);
pub const UDP_PORT: u16 = 7645;

pub type StringVecIp = Arc<Mutex<HashMap<String, Vec<(IpAddr, bool)>>>>;
pub type IpVecString = Arc<Mutex<HashMap<IpAddr, Vec<(String, bool)>>>>;

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestType {
    Scan,
    ScanAfterRestart,
    ScanResponse(Vec<String>),
}

pub fn mark_unmark<K, V>(
    files_list: &mut Arc<Mutex<HashMap<K, Vec<(V, bool)>>>>,
    key: K,
    val: V,
    action: bool,
) where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    V: std::cmp::PartialEq,
{
    files_list
        .lock()
        .unwrap()
        .get_mut(&key)
        .unwrap()
        .iter_mut()
        .find(|x| x.0 == val)
        .unwrap()
        .1 = action;
}

pub fn push_to_hash_map<K, V>(files_list: &Arc<Mutex<HashMap<K, Vec<V>>>>, key: K, val: Vec<V>)
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
{
    match files_list.lock().unwrap().entry(key) {
        Entry::Occupied(mut container) => {
            for el in val {
                container.get_mut().push(el);
            }
        }
        Entry::Vacant(v) => {
            v.insert(val);
        }
    }
}

pub fn remove_from_hash_map(
    files_list: &Arc<Mutex<HashMap<String, Vec<IpAddr>>>>,
    file_name: &str,
    remote_addr: &IpAddr,
) {
    let index = files_list
        .lock()
        .unwrap()
        .get(file_name)
        .unwrap()
        .iter()
        .position(|x| x == remote_addr)
        .unwrap();
    files_list
        .lock()
        .unwrap()
        .get_mut(file_name)
        .unwrap()
        .remove(index);
}

pub fn recv_send_overall_code<T1: std::io::Read, T2: std::io::Write>(
    bs_1: &mut T1, // убрать ссылку у stream
    bs_2: &mut T2,
) -> Result<(), io::Error> {
    loop {
        let mut buffer = [0; 102_400];
        let n = bs_1.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bs_2.write_all(&buffer[..n])?;
        bs_2.flush()?;
    }
    Ok(())
}

pub fn recv_file_throw_errors_level_1(
    remote_addr: IpAddr,
    full_file_name: String,
    foreign_files_to_download_list: &mut IpVecString,
) -> Result<(), io::Error> {
    let file_name = extract_file_name(full_file_name.clone());

    let mut stream = TcpStream::connect(SocketAddr::new(remote_addr, PORT + 1))?;
    stream.write_all(extract_file_name(file_name.clone()).as_bytes())?;
    stream.flush()?;

    let mut file = File::create(&full_file_name)?;
    mark_unmark(
        foreign_files_to_download_list,
        remote_addr,
        file_name.clone(),
        true,
    );
    if let Err(err) = recv_send_overall_code::<TcpStream, File>(&mut stream, &mut file) {
        mark_unmark(
            foreign_files_to_download_list,
            remote_addr,
            file_name.clone(),
            false,
        );
        fs::remove_file(full_file_name.clone())?;
        return Err(err);
    }
    mark_unmark(
        foreign_files_to_download_list,
        remote_addr,
        file_name.clone(),
        false,
    );
    Ok(())
}

pub fn recv_file(
    pool: &Arc<Mutex<ThreadPool>>,
    remote_addr: IpAddr,
    full_file_name: String,
    mut foreign_files_to_download_list: IpVecString,
) {
    pool.lock().unwrap().execute(move || {
        if let Err(err) = recv_file_throw_errors_level_1(
            remote_addr,
            full_file_name,
            &mut foreign_files_to_download_list,
        ) {
            eprint!("Occurs error: {}", err);
        };
    });
}

pub fn send_file_throw_errors_level_1(
    full_file_name: String,
    my_files_to_share_list: &mut StringVecIp,
    remote_addr: IpAddr,
    mut stream: TcpStream,
) -> Result<(), io::Error> {
    let mut file = File::open(&full_file_name)?;

    mark_unmark(
        my_files_to_share_list,
        full_file_name.clone(),
        remote_addr,
        true,
    );
    if let Err(err) = recv_send_overall_code::<File, TcpStream>(&mut file, &mut stream) {
        mark_unmark(
            my_files_to_share_list,
            full_file_name.clone(),
            remote_addr,
            false,
        );
        return Err(err);
    }
    mark_unmark(
        my_files_to_share_list,
        full_file_name.clone(),
        remote_addr,
        false,
    );
    Ok(())
}

pub fn send_file(
    pool: &Arc<Mutex<ThreadPool>>,
    full_file_name: String,
    mut my_files_to_share_list: StringVecIp,
    stream: TcpStream,
) {
    pool.lock().unwrap().execute(move || {
        if let Err(err) = send_file_throw_errors_level_1(
            full_file_name,
            &mut my_files_to_share_list,
            stream.peer_addr().unwrap().ip(),
            stream,
        ) {
            eprint!("Occurs error: {}", err);
        };
    });
}

pub fn extract_file_name(mut full_file_name: String) -> String {
    match full_file_name.rfind('/') {
        None => full_file_name,
        Some(i) => {
            let mut file_name = full_file_name.split_off(i);
            file_name.remove(0);
            file_name
        }
    }
}
