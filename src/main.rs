use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::net::IpAddr;
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
const MULTI_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);
const PORT: u16 = 7645;

#[derive(Serialize, Deserialize, Debug)]
enum Command_Type{
    scan,
    ls,
    share
}

#[derive(Serialize, Deserialize, Debug)]
enum Request_type{
    scan,
    scan_response(String)
}

fn from_u8vec_to_String(buffer: &Vec<u8>, message_len: usize) -> String{
    let mut res=String::new();
    let mut i=0;
    while i < message_len{
        let el=buffer[i];
        res.push(el as char);
        i=i+1;
    };
    res
}

#[cfg(windows)]
fn get_ips_v4() -> IpAddr{//self put off
    ipconfig::get_adapters().unwrap()[0].ip_addresses()[1]
}

fn run_client_listener(
    my_files_toShare_list: Arc<Mutex<Vec<String>>>,
    foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>
)
{
    thread::spawn(move|| {
        let listener = TcpListener::bind("localhost:1234").unwrap();
        let mut streamWrapped=listener.incoming();
        let mut stream = streamWrapped.next().unwrap().unwrap();
        loop{
            let mut buffer = vec![0; 4096];
            let message_len = stream.read(& mut buffer);

            let command: Command_Type = serde_json::from_str(
                &from_u8vec_to_String(&buffer, message_len.unwrap())
            ).unwrap();

            match command{
                Command_Type::scan=>{
                    let serialized_Request_type = serde_json::to_string(&Request_type::scan).unwrap();
                    let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).unwrap();
                    socket.send_to(serialized_Request_type.as_bytes(), (MULTI_ADDR, PORT)).unwrap();
                }
                Command_Type::share=>{
                    my_files_toShare_list.lock().unwrap().push("".to_string());
                }
                Command_Type::ls=>{
                    let response_to_client=serde_json::to_string(
                        &*foreign_files_toDownload_list.lock().unwrap()
                    ).unwrap();
                    stream.write(response_to_client.as_bytes());
                }
            }
        }
    });
}

fn run_another_daemon_listener(
    my_files_toShare_list: Arc<Mutex<Vec<String>>>,
    foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>
)
{
    let listener_another_daemon = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), PORT), ).unwrap();
    listener_another_daemon.join_multicast_v4(&MULTI_ADDR, &Ipv4Addr::new(0, 0, 0, 0));

    loop{
        let mut buffer = vec![0; 4096];
        let (len, mut remote_addr) = listener_another_daemon.recv_from(&mut buffer).unwrap();
        if remote_addr.ip().is_ipv4() &&  remote_addr.ip() == get_ips_v4() { continue; }

        let request_type = serde_json::from_str(
            &from_u8vec_to_String(&buffer, len)
        ).unwrap();

        match request_type {
            Request_type::scan=>{
                let request_type=Request_type::scan_response(
                    serde_json::to_string(&my_files_toShare_list.lock().unwrap()[..]).unwrap()
                );
                let to_send_ser=serde_json::to_string(&request_type).unwrap();

                remote_addr.set_port(PORT);
                UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).unwrap().send_to(
                    to_send_ser.as_bytes(), remote_addr
                );
            }
            Request_type::scan_response(response)=>{
                let mut vec_to_map:Vec<String>=serde_json::from_str(&response).unwrap();

                match foreign_files_toDownload_list.lock().unwrap().get_mut(&remote_addr.ip()){
                    Some(v)=>{
                        for el in vec_to_map{
                            v.push(el);
                        }
                    }
                    None=>{
                        foreign_files_toDownload_list.lock().unwrap().insert(
                            remote_addr.ip(),
                            vec_to_map
                        );
                    }
                }
            }
        }
    }
}

fn run(){
    let my_files_toShare_list=Arc::new(Mutex::new(Vec::new()));
    let foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>=Arc::new(Mutex::new(HashMap::new()));

    run_client_listener(my_files_toShare_list.clone(), foreign_files_toDownload_list.clone());
    run_another_daemon_listener(my_files_toShare_list.clone(), foreign_files_toDownload_list.clone());
}



fn main() {
    run();
}
