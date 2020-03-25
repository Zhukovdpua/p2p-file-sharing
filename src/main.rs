use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::thread;
use std::net::IpAddr;
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use threadpool::ThreadPool;
use std::thread::JoinHandle;
use std::fs::File;
use std::collections::hash_map::Entry;

const MULTI_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 123);
const PORT: u16 = 7645;

type load_files = Arc<Mutex<HashMap<String, Vec<IpAddr>>>>;
type share_files = load_files;

#[derive(Serialize, Deserialize, Debug)]
enum Command_Type{
    share(String),
    scan,
    ls,
    download(String, String),
    status
}

#[derive(Serialize, Deserialize, Debug)]
enum Request_type{
    scan,
    scan_response(String),
    download(String),
    download_response(String)
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

fn send_request_to_another_daemon(request_type: Request_type) {
    let serialized_Request_type = serde_json::to_string(&request_type).unwrap();
    UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).unwrap().send_to(
        serialized_Request_type.as_bytes(), (MULTI_ADDR, PORT)
    );
}

fn send_response(request_type: Request_type, mut remote_addr: SocketAddr){
    let to_send_ser=serde_json::to_string(&request_type).unwrap();
    remote_addr.set_port(PORT);
    UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).unwrap().send_to(
        to_send_ser.as_bytes(), remote_addr
    );
}

fn push_to_HashMap<K, V>(files_list: &Arc<Mutex<HashMap<K, Vec<V>>>>, key: K, val: Vec<V>)
    where
        K: std::cmp::Eq,
        K: std::hash::Hash
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

fn run_client_listener(
    my_files_toShare_list: Arc<Mutex<Vec<String>>>,
    foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>,
    save_path_list: Arc<Mutex<HashMap<String, String>>>,
    downloading_files_list: load_files,
    sharing_files_list: share_files
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
                Command_Type::share(file_name)=>{
                    my_files_toShare_list.lock().unwrap().push("".to_string());
                }
                Command_Type::scan=>{
                    send_request_to_another_daemon(Request_type::scan);
                }
                Command_Type::ls=>{
                    let response_to_client=serde_json::to_string(
                        &*foreign_files_toDownload_list.lock().unwrap()
                    ).unwrap();
                    stream.write(response_to_client.as_bytes());
                }
                Command_Type::download(file_name, save_path)=>{
                    save_path_list.lock().unwrap().insert(file_name.clone(), save_path);
                    send_request_to_another_daemon(Request_type::download(file_name));
                }
                Command_Type::status => {
                    let tuple_to_send = (&(*downloading_files_list.lock().unwrap()), &(*sharing_files_list.lock().unwrap()));
                    let response_to_client=serde_json::to_string(&tuple_to_send).unwrap();
                    stream.write(response_to_client.as_bytes());
                }
            }
        }
    });
}

fn create_TCP_chanel_to_download (
    pool: Arc<Mutex<ThreadPool>>,
    file_name: String,
    sharing_files_list: share_files,
    val: IpAddr
) -> JoinHandle<()>
{
    thread::spawn(move|| {
        let listener = TcpListener::bind("localhost:1234").unwrap();
        let mut streamWrapped=listener.incoming();
        let mut stream = streamWrapped.next().unwrap().unwrap();
        pool.lock().unwrap().execute(move|| {
            let mut file = File::open(file_name.clone()).unwrap();
            let mut buffer=Vec::new();
            let n = file.read_to_end(&mut buffer).unwrap();
            let mut tmp_v=Vec::new();
            tmp_v.push(val);
            push_to_HashMap(&sharing_files_list, file_name.clone(), tmp_v);
            stream.write(&buffer[..n]).unwrap();

            let index = sharing_files_list.lock().unwrap().get(&file_name).unwrap().iter().position(|x| *x == val).unwrap(); // change to retain if what
            sharing_files_list.lock().unwrap().get_mut(&file_name).unwrap().remove(index);
        });
    })
}

fn start_download(
    pool: Arc<Mutex<ThreadPool>>,
    remote_addr: SocketAddr,
    file_name: String,
    save_path_list: Arc<Mutex<HashMap<String, String>>>,
    downloading_files_list: load_files
)
{
    pool.lock().unwrap().execute(move|| {
        let mut stream = TcpStream::connect(remote_addr).unwrap();
        let mut buffer=Vec::new();
        let mut tmp_v=Vec::new();
        tmp_v.push(remote_addr.ip());
        push_to_HashMap(&downloading_files_list, file_name.clone(), tmp_v);
        let n = stream.read_to_end(& mut buffer).unwrap();
        let index = downloading_files_list.lock().unwrap().get(&file_name).unwrap().iter().position(
            |x| *x == remote_addr.ip()
        ).unwrap(); // change to retain if what
        downloading_files_list.lock().unwrap().get_mut(&file_name).unwrap().remove(index);

        let save_path= save_path_list.lock().unwrap().remove(&file_name).unwrap();
        let mut file = File::create( save_path + file_name.as_str()).unwrap();
        file.write(&buffer[..n]);
    });
}

fn run_another_daemon_listener(
    my_files_toShare_list: Arc<Mutex<Vec<String>>>,
    foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>,
    save_path_list: Arc<Mutex<HashMap<String, String>>>,
    downloading_files_list: load_files,
    sharing_files_list: share_files
)
{
    let listener_another_daemon = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), PORT), ).unwrap();
    listener_another_daemon.join_multicast_v4(&MULTI_ADDR, &Ipv4Addr::new(0, 0, 0, 0));
    let pool = Arc::new(Mutex::new(ThreadPool::new(4))); // hardware conc

    loop {
        let mut buffer = vec![0; 4096];
        let (len, mut remote_addr) = listener_another_daemon.recv_from(&mut buffer).unwrap();
        if remote_addr.ip().is_ipv4() &&  remote_addr.ip() == get_ips_v4() { continue; }

        let request_type = serde_json::from_str(
            &from_u8vec_to_String(&buffer, len)
        ).unwrap();

        match request_type {
            Request_type::scan=>{
                send_response(Request_type::scan_response(
                    serde_json::to_string(&my_files_toShare_list.lock().unwrap()[..]).unwrap()), remote_addr
                );
            }
            Request_type::scan_response(response)=>{
                let mut vec_to_map:Vec<String>=serde_json::from_str(&response).unwrap();
                push_to_HashMap(&foreign_files_toDownload_list, remote_addr.ip(), vec_to_map);
            }
            Request_type::download(file_name)=>{
                for file_name_el in  my_files_toShare_list.lock().unwrap().iter(){
                    if file_name_el == &file_name{
                        let handle = create_TCP_chanel_to_download(
                            pool.clone(),file_name.clone(), sharing_files_list.clone(), remote_addr.ip()
                        );
                        send_response(Request_type::download_response(file_name.clone()), remote_addr);
                        handle.join();
                        break;
                    }
                }
            }
            Request_type::download_response(file_name) => {
                start_download(pool.clone(),
                               remote_addr,
                               file_name,
                               save_path_list.clone(),
                               downloading_files_list.clone()
                );
            }
        }
    }
}

fn run(){
    let my_files_toShare_list=Arc::new(Mutex::new(Vec::new()));
    let foreign_files_toDownload_list:Arc<Mutex<HashMap<IpAddr, Vec<String>>>>=Arc::new(Mutex::new(HashMap::new()));
    let save_path_list:Arc<Mutex<HashMap<String, String>>>=Arc::new(Mutex::new(HashMap::new()));
    let downloading_files_list  = Arc::new(Mutex::new(HashMap::new()));
    let sharing_files_list  = Arc::new(Mutex::new(HashMap::new()));

    run_client_listener(
        my_files_toShare_list.clone(),
        foreign_files_toDownload_list.clone(),
        save_path_list.clone(),
        downloading_files_list.clone(),
        sharing_files_list.clone()

    );
    run_another_daemon_listener(
        my_files_toShare_list.clone(),
        foreign_files_toDownload_list.clone(),
        save_path_list.clone(),
        downloading_files_list.clone(),
        sharing_files_list.clone()
    );
}

fn main() {
    run();
}