use p2p_file_sharing::client_listen::listen_to_client;
use p2p_file_sharing::daemon_multicast_listen::listen_to_other_demons_via_multicast;
use p2p_file_sharing::daemon_tcp_listen::listen_to_other_demons_via_tcp;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
    let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));

    listen_to_client(
        my_files_to_share_list.clone(),
        foreign_files_to_download_list.clone(),
    );

    listen_to_other_demons_via_tcp(my_files_to_share_list.clone());
    listen_to_other_demons_via_multicast(&my_files_to_share_list, &foreign_files_to_download_list);
}
