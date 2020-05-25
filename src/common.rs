use p2p_file_sharing_enum_commands::PORT;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadData {
    pub file_name: String,
    peer_count: u64,
    index: u64,
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

fn recv_overall<T1: std::io::Read, T2: std::io::Write>(
    bs_1: &mut T1,
    bs_2: &mut T2,
) -> Result<(), io::Error> {
    loop {
        let mut buffer = [0; 100_000];
        let n = bs_1.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bs_2.write_all(&buffer[..n])?;
        bs_2.flush()?;
    }
    Ok(())
}

fn send(
    file: &mut File,
    stream: &mut TcpStream,
    file_size: u64,
    peer_count: u64,
    index: u64,
) -> Result<(), io::Error> {
    let mut size_current_block = if index == peer_count {
        file_size - file_size / peer_count * (index - 1)
    } else {
        file_size / peer_count
    } as usize;

    while size_current_block > 0 {
        let mut buffer = Vec::new();
        buffer.resize(min(size_current_block, 100_000), 0);
        let n = file.read(&mut buffer)?;
        size_current_block -= n;

        stream.write_all(&buffer)?;
        stream.flush()?;
    }
    Ok(())
}

fn build_file(out_file: &mut File, peer_count: u64) -> Result<(), io::Error> {
    for i in 1..peer_count + 1 {
        let temp_file_name = "out_".to_owned() + &i.to_string();
        let mut in_file = File::open(&temp_file_name)?;
        if let Err(err) = recv_overall::<File, File>(&mut in_file, out_file) {
            fs::remove_file(&temp_file_name)?;
            return Err(err);
        }
        fs::remove_file(&temp_file_name)?;
    }
    Ok(())
}

fn recv_file_throw_errors_level_1(
    remote_addr: IpAddr,
    full_file_name: String,
    foreign_files_to_download_list: &mut IpVecString,
    peer_count: u64,
    index: u64,
    ended_count: &Arc<AtomicU64>,
) -> Result<(), io::Error> {
    let file_name = extract_file_name(full_file_name.clone());

    let mut stream = TcpStream::connect(SocketAddr::new(remote_addr, PORT + 1))?;
    stream.write_all(
        serde_json::to_string(&DownloadData {
            file_name: file_name.clone(),
            peer_count,
            index,
        })?
        .as_bytes(),
    )?;
    stream.flush()?;

    let temp_file_name = "out_".to_owned() + &index.to_string();
    let mut file = File::create(temp_file_name.clone())?;
    mark_unmark(
        foreign_files_to_download_list,
        remote_addr,
        file_name.clone(),
        true,
    );
    if let Err(err) = recv_overall::<TcpStream, File>(&mut stream, &mut file) {
        mark_unmark(
            foreign_files_to_download_list,
            remote_addr,
            file_name.clone(),
            false,
        );
        fs::remove_file(temp_file_name)?;
        return Err(err);
    }

    if ended_count.fetch_add(1, Ordering::SeqCst) == peer_count - 1 {
        let mut file = match File::create(&full_file_name) {
            Ok(file) => file,
            Err(err) => {
                mark_unmark(
                    foreign_files_to_download_list,
                    remote_addr,
                    file_name.clone(),
                    false,
                );
                return Err(err);
            }
        };

        if let Err(err) = build_file(&mut file, peer_count) {
            mark_unmark(
                foreign_files_to_download_list,
                remote_addr,
                file_name.clone(),
                false,
            );
            fs::remove_file(&full_file_name)?;
            return Err(err);
        }
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
    remote_addrs: Vec<IpAddr>,
    full_file_name: String,
    foreign_files_to_download_list: &IpVecString,
) {
    let ended_count = Arc::new(AtomicU64::new(0));
    for i in 0..remote_addrs.len() {
        let to_thread_addr = remote_addrs[i];
        let to_thread_len = remote_addrs.len() as u64;
        let full_file_name = full_file_name.clone();
        let mut foreign_files_to_download_list = foreign_files_to_download_list.clone();
        let ended_count = ended_count.clone();

        pool.lock().unwrap().execute(move || {
            if let Err(err) = recv_file_throw_errors_level_1(
                to_thread_addr,
                full_file_name.clone(),
                &mut foreign_files_to_download_list,
                to_thread_len,
                (i + 1) as u64,
                &ended_count,
            ) {
                eprint!("Occurs error: {}", err);
            };
        })
    }
}

pub fn send_file_throw_errors_level_1(
    full_file_name: String,
    my_files_to_share_list: &mut StringVecIp,
    remote_addr: IpAddr,
    mut stream: TcpStream,
    peer_count: u64,
    index: u64,
) -> Result<(), io::Error> {
    let mut file = File::open(&full_file_name)?;
    let file_size = fs::metadata(&full_file_name)?.len();
    file.seek(SeekFrom::Start(file_size / peer_count * (index - 1)))?;

    mark_unmark(
        my_files_to_share_list,
        full_file_name.clone(),
        remote_addr,
        true,
    );
    if let Err(err) = send(&mut file, &mut stream, file_size, peer_count, index) {
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
    download_data: DownloadData,
) {
    pool.lock().unwrap().execute(move || {
        if let Err(err) = send_file_throw_errors_level_1(
            full_file_name,
            &mut my_files_to_share_list,
            stream.peer_addr().unwrap().ip(),
            stream,
            download_data.peer_count,
            download_data.index,
        ) {
            eprint!("Occurs error: {}", err);
        };
    });
}
