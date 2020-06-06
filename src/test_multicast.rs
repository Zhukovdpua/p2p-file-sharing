use crate::common::{MULTI_ADDR, UDP_PORT};
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_multicast() {
    assert!(MULTI_ADDR.is_multicast());

    multicast_listener();

    UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))
        .unwrap()
        .send_to(b"TEST", (MULTI_ADDR, UDP_PORT))
        .unwrap();
}

fn multicast_listener() {
    let recv_barrier = Arc::new(Barrier::new(2));
    let send_barrier = Arc::clone(&recv_barrier);

    thread::spawn(move || {
        let listener_another_daemon =
            UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), UDP_PORT)).unwrap();
        listener_another_daemon
            .join_multicast_v4(&MULTI_ADDR, &Ipv4Addr::new(0, 0, 0, 0))
            .unwrap();

        let mut buffer = vec![0; 40960];
        recv_barrier.wait();
        let (n, _) = listener_another_daemon.recv_from(&mut buffer).unwrap();

        assert_eq!(String::from_utf8_lossy(&buffer[..n]), String::from("TEST"));
    });
    send_barrier.wait();
}
