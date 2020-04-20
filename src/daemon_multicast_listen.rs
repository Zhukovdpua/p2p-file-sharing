use crate::common::{
    extract_file_name, push_to_hash_map, IpVecString, RequestType, StringVecIp, MULTI_ADDR,
    UDP_PORT,
};

use ipconfig::get_adapters;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

#[cfg(windows)]
fn get_ips_v4() -> Result<(IpAddr, IpAddr, IpAddr, IpAddr), ipconfig::error::Error> {
    Ok((
        get_adapters()?[0].ip_addresses()[1],
        get_adapters()?[1].ip_addresses()[1],
        get_adapters()?[2].ip_addresses()[1],
        get_adapters()?[3].ip_addresses()[1],
    ))
}

#[cfg(unix)]
extern crate machine_ip;

#[cfg(unix)]
fn get_ips_v4() -> Result<IpAddr, ipconfig::error::Error> {
    machine_ip::get()?
}

fn filter_files_to_send(my_files_to_share_list: &StringVecIp, remote_addr: IpAddr) -> Vec<String> {
    let mut files_to_send = Vec::new();
    for (key, val) in my_files_to_share_list.lock().unwrap().iter_mut() {
        if let None = val.iter().find(|x| x.0 == remote_addr) {
            files_to_send.push(extract_file_name(key.clone()));
            val.push((remote_addr, false));
        }
    }
    files_to_send
}

fn non_filter_files_to_send(
    my_files_to_share_list: &StringVecIp,
    remote_addr: IpAddr,
) -> Vec<String> {
    let mut files_to_send = Vec::new();
    for (key, val) in my_files_to_share_list.lock().unwrap().iter_mut() {
        if let None = val.iter().find(|x| x.0 == remote_addr) {
            val.push((remote_addr, false));
        }
        files_to_send.push(extract_file_name(key.clone()));
    }
    files_to_send
}

fn send_response_to_daemon(
    files_to_send: Vec<String>,
    mut remote_addr: SocketAddr,
) -> Result<(), io::Error> {
    let serialized = serde_json::to_string(&RequestType::ScanResponse(files_to_send))?;
    remote_addr.set_port(UDP_PORT);
    UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))?.send_to(serialized.as_bytes(), remote_addr)?;
    Ok(())
}

fn listen_to_other_demons_via_multicast_throw_errors(
    my_files_to_share_list: &StringVecIp,
    foreign_files_to_download_list: &IpVecString,
    buffer: &[u8],
    remote_addr: SocketAddr,
) -> Result<(), io::Error> {
    match get_ips_v4() {
        Ok(self_ip) => {
            if remote_addr.ip() == self_ip.0
                || remote_addr.ip() == self_ip.1
                || remote_addr.ip() == self_ip.2
                || remote_addr.ip() == self_ip.3
            {
                return Ok(());
            }
        }
        Err(err) => {
            eprint!("Occurs error: {}", err);
            return Ok(());
        }
    }

    let request_type = serde_json::from_str(&String::from_utf8_lossy(buffer))?;
    match request_type {
        RequestType::Scan => {
            send_response_to_daemon(
                filter_files_to_send(&my_files_to_share_list, remote_addr.ip()),
                remote_addr,
            )?;
        }
        RequestType::ScanAfterRestart => {
            send_response_to_daemon(
                non_filter_files_to_send(&my_files_to_share_list, remote_addr.ip()),
                remote_addr,
            )?;
        }
        RequestType::ScanResponse(response) => {
            let mut tmp_v = Vec::new();
            for el in response.iter() {
                tmp_v.push((el.clone(), false));
            }
            push_to_hash_map(&foreign_files_to_download_list, remote_addr.ip(), tmp_v);
        }
    }
    Ok(())
}

pub fn listen_to_other_demons_via_multicast(
    my_files_to_share_list: &StringVecIp,
    foreign_files_to_download_list: &IpVecString,
) {
    let listener_another_daemon = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), UDP_PORT)).unwrap();
    listener_another_daemon
        .join_multicast_v4(&MULTI_ADDR, &Ipv4Addr::new(0, 0, 0, 0))
        .unwrap();

    loop {
        let mut buffer = vec![0; 40960];
        let (n, remote_addr) = match listener_another_daemon.recv_from(&mut buffer) {
            Ok(result) => result,
            Err(err) => {
                eprint!("Occurs error: {}", err);
                continue;
            }
        };

        if let Err(err) = listen_to_other_demons_via_multicast_throw_errors(
            &my_files_to_share_list,
            &foreign_files_to_download_list,
            &buffer[..n],
            remote_addr,
        ) {
            eprint!("Occurs error: {}", err);
        }
    }
}
