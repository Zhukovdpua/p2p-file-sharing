extern crate num_cpus;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use p2p_file_sharing::client_listen::listen_to_client;
use p2p_file_sharing::daemon_tcp_listen::listen_to_other_demons_via_tcp;
use p2p_file_sharing::daemon_multicast_listen::listen_to_other_demons_via_multicast;

fn main() {
    let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
    let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
    let pool = Arc::new(Mutex::new(ThreadPool::new(num_cpus::get())));

    listen_to_client(
        my_files_to_share_list.clone(),
        foreign_files_to_download_list.clone(),
        pool.clone(),
    );
    listen_to_other_demons_via_tcp(my_files_to_share_list.clone(), pool);
    listen_to_other_demons_via_multicast(&my_files_to_share_list, &foreign_files_to_download_list);
}
