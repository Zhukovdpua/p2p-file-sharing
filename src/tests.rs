use crate::client_listen::filter_peers;
use crate::client_listen::find_file;
use crate::client_listen::remove_tuple_to_ls_response;
use crate::client_listen::select_downloading_files_to_send;
use crate::client_listen::select_sharing_files_to_send;
use crate::common::extract_file_name;
use crate::common::mark_unmark;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

#[test]
fn extract_file_name_test() {
    assert_eq!(
        extract_file_name(String::from("D:/file.txt")),
        String::from("file.txt")
    );
    assert_eq!(
        extract_file_name(String::from("D://file.txt")),
        String::from("file.txt")
    );
    assert_eq!(
        extract_file_name(String::from("D:file.txt")),
        String::from("D:file.txt")
    );

    assert_eq!(
        extract_file_name(String::from("C:/Documents/work/resume.pdf")),
        String::from("resume.pdf")
    );
    assert_eq!(
        extract_file_name(String::from("movie.mp4")),
        String::from("movie.mp4")
    );
}

#[test]
fn find_file_test() {
    {
        let file_1_name = String::from("key.txt");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list
            .lock()
            .unwrap()
            .insert(remote_addr, vec![(file_1_name.clone(), false)]);
        let result = vec![remote_addr];

        assert_eq!(
            find_file(&foreign_files_to_download_list, file_1_name),
            Some(result)
        );
    }

    {
        let file_1_name = String::from("key.txt");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list
            .lock()
            .unwrap()
            .insert(remote_addr, vec![(file_1_name.clone(), false)]);

        assert_eq!(
            find_file(&foreign_files_to_download_list, String::from("resume.pdf")),
            None
        );
    }

    {
        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        assert_eq!(
            find_file(&foreign_files_to_download_list, String::from("resume.pdf")),
            None
        );
    }
    {
        let file_1_name = String::from("Serious_sam.exe");
        let file_2_name = String::from("film.mp4");
        let file_3_name = String::from("resume.pdf");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr,
            vec![
                (file_1_name.clone(), false),
                (file_2_name.clone(), false),
                (file_3_name.clone(), false),
            ],
        );

        let result = vec![remote_addr];

        assert_eq!(
            find_file(&foreign_files_to_download_list, file_2_name),
            Some(result)
        );
    }

    {
        let file_1_name = String::from("Serious_sam.exe");
        let file_2_name = String::from("film.mp4");
        let file_3_name = String::from("resume.pdf");

        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(231, 0, 2, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_1,
            vec![(file_1_name.clone(), false), (file_2_name.clone(), false)],
        );

        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_2,
            vec![
                (file_3_name.clone(), false),
                (file_2_name.clone(), false),
                (file_1_name.clone(), false),
            ],
        );

        let f = find_file(&foreign_files_to_download_list, file_2_name);
        assert_eq!(
            (f == Some(vec![remote_addr_1, remote_addr_2])
                || f == Some(vec![remote_addr_2, remote_addr_1])),
            true
        );
        assert_eq!(f.unwrap().len(), 2);
    }

    {
        let file_1_name = String::from("key.txt");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list
            .lock()
            .unwrap()
            .insert(remote_addr, vec![(file_1_name.clone(), true)]);

        assert_eq!(
            find_file(&foreign_files_to_download_list, file_1_name),
            None
        );
    }

    {
        let file_1_name = String::from("Serious_sam.exe");
        let file_2_name = String::from("film.mp4");
        let file_3_name = String::from("resume.pdf");

        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(231, 0, 2, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_1,
            vec![(file_1_name.clone(), false), (file_2_name.clone(), false)],
        );

        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_2,
            vec![
                (file_3_name.clone(), false),
                (file_2_name.clone(), true),
                (file_1_name.clone(), false),
            ],
        );

        let f = find_file(&foreign_files_to_download_list, file_2_name);
        assert_eq!(f, None);
    }
}

#[test]
fn remove_tuple_to_ls_response_test() {
    {
        let file_1_name = String::from("key.txt");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list
            .lock()
            .unwrap()
            .insert(remote_addr, vec![(file_1_name.clone(), false)]);

        let mut result: HashMap<IpAddr, Vec<String>> = HashMap::new();
        result.insert(remote_addr, vec![file_1_name.clone()]);

        assert_eq!(
            remove_tuple_to_ls_response(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/Games/Serious_sam.exe");
        let file_2_name = String::from("D:/free/film.mp4");
        let file_3_name = String::from("C:/resume.pdf");
        let file_4_name = String::from("crypt.txt");
        let file_5_name = String::from("key.txt");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr,
            vec![
                (file_1_name.clone(), false),
                (file_2_name.clone(), false),
                (file_3_name.clone(), true),
                (file_4_name.clone(), true),
                (file_5_name.clone(), false),
            ],
        );

        let mut result: HashMap<IpAddr, Vec<String>> = HashMap::new();
        result.insert(
            remote_addr,
            vec![
                file_1_name.clone(),
                file_2_name.clone(),
                file_3_name.clone(),
                file_4_name.clone(),
                file_5_name.clone(),
            ],
        );

        assert_eq!(
            remove_tuple_to_ls_response(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/Games/Serious_sam.exe");
        let file_2_name = String::from("D:/free/film.mp4");
        let file_3_name = String::from("C:/resume.pdf");
        let file_4_name = String::from("crypt.txt");
        let file_5_name = String::from("key.txt");
        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(231, 2, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_1,
            vec![
                (file_1_name.clone(), false),
                (file_2_name.clone(), false),
                (file_3_name.clone(), true),
            ],
        );

        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_2,
            vec![(file_4_name.clone(), false), (file_5_name.clone(), true)],
        );

        let mut result: HashMap<IpAddr, Vec<String>> = HashMap::new();
        result.insert(
            remote_addr_1,
            vec![
                file_1_name.clone(),
                file_2_name.clone(),
                file_3_name.clone(),
            ],
        );

        result.insert(
            remote_addr_2,
            vec![file_4_name.clone(), file_5_name.clone()],
        );

        assert_eq!(
            remove_tuple_to_ls_response(&foreign_files_to_download_list),
            result
        );
    }
}

#[test]
fn mark_unmark_share_test() {
    {
        let file_1_name = String::from("D:/film/movie.mkv");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let mut my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
        my_files_to_share_list
            .lock()
            .unwrap()
            .insert(file_1_name.clone(), vec![(remote_addr, false)]);

        mark_unmark(
            &mut my_files_to_share_list,
            file_1_name.clone(),
            remote_addr,
            true,
        );
        assert_eq!(
            my_files_to_share_list
                .lock()
                .unwrap()
                .get(&file_1_name)
                .unwrap()
                .iter()
                .find(|&&x| x == (remote_addr, true)),
            Some(&(remote_addr, true))
        );

        mark_unmark(
            &mut my_files_to_share_list,
            file_1_name.clone(),
            remote_addr,
            false,
        );
        assert_eq!(
            my_files_to_share_list
                .lock()
                .unwrap()
                .get(&file_1_name)
                .unwrap()
                .iter()
                .find(|&&x| x == (remote_addr, false)),
            Some(&(remote_addr, false))
        );
    }
}

#[test]
fn mark_unmark_downloading_test() {
    {
        let file_1_name = String::from("key.txt");
        let file_2_name = String::from("cpp_book.djvu");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let mut foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr,
            vec![(file_1_name.clone(), false), (file_2_name.clone(), false)],
        );

        mark_unmark(
            &mut foreign_files_to_download_list,
            remote_addr,
            file_2_name.clone(),
            true,
        );
        assert_eq!(
            foreign_files_to_download_list
                .lock()
                .unwrap()
                .get(&remote_addr)
                .unwrap()
                .iter()
                .find(|&x| x == &(file_2_name.clone(), true)),
            Some(&(file_2_name.clone(), true))
        );

        mark_unmark(
            &mut foreign_files_to_download_list,
            remote_addr,
            file_2_name.clone(),
            false,
        );
        assert_eq!(
            foreign_files_to_download_list
                .lock()
                .unwrap()
                .get(&remote_addr)
                .unwrap()
                .iter()
                .find(|&x| x == &(file_2_name.clone(), false)),
            Some(&(file_2_name.clone(), false))
        );
    }
}

#[test]
fn select_downloading_files_to_send_test() {
    {
        let file_name = String::from("D:/film/movie.mkv");

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)),
            vec![(file_name, false)],
        );

        let result: HashMap<String, Vec<IpAddr>> = HashMap::new();

        assert_eq!(
            select_downloading_files_to_send(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_name = String::from("D:/film/movie.mkv");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list
            .lock()
            .unwrap()
            .insert(remote_addr, vec![(file_name.clone(), true)]);

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_name, vec![remote_addr]);

        assert_eq!(
            select_downloading_files_to_send(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/film/movie.mkv");
        let file_2_name = String::from("C:/docs/resume.pdf");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr,
            vec![(file_1_name.clone(), false), (file_2_name.clone(), true)],
        );

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_2_name, vec![remote_addr]);

        assert_eq!(
            select_downloading_files_to_send(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/film/movie.mkv");
        let file_2_name = String::from("C:/docs/resume.pdf");
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr,
            vec![(file_1_name.clone(), true), (file_2_name.clone(), true)],
        );

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_1_name, vec![remote_addr]);
        result.insert(file_2_name, vec![remote_addr]);

        assert_eq!(
            select_downloading_files_to_send(&foreign_files_to_download_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/film/movie.mkv");
        let file_2_name = String::from("C:/docs/resume.pdf");
        let file_3_name = String::from("D:/free/film.mp4");
        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12));

        let foreign_files_to_download_list = Arc::new(Mutex::new(HashMap::new()));
        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_1,
            vec![(file_1_name.clone(), false), (file_2_name.clone(), true)],
        );

        foreign_files_to_download_list.lock().unwrap().insert(
            remote_addr_2,
            vec![(file_2_name.clone(), true), (file_3_name.clone(), false)],
        );

        let mut result_var_1: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result_var_1.insert(file_2_name.clone(), vec![remote_addr_1, remote_addr_2]);

        let mut result_var_2: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result_var_2.insert(file_2_name, vec![remote_addr_2, remote_addr_1]);

        let x = select_downloading_files_to_send(&foreign_files_to_download_list);
        assert_eq!((x == result_var_1 || x == result_var_2), true);
    }
}

#[test]
fn select_sharing_files_to_send_test() {
    {
        let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
        let file_name = String::from("D:/film/movie.mkv");

        my_files_to_share_list.lock().unwrap().insert(
            file_name.clone(),
            vec![
                (IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)), false),
                (IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12)), false),
            ],
        );

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_name, vec![]);

        assert_eq!(
            select_sharing_files_to_send(&my_files_to_share_list),
            result
        );
    }

    {
        let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
        let file_name = String::from("D:/film/movie.mkv");

        my_files_to_share_list
            .lock()
            .unwrap()
            .insert(file_name.clone(), vec![]);

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_name, vec![]);

        assert_eq!(
            select_sharing_files_to_send(&my_files_to_share_list),
            result
        );
    }

    {
        let my_files_to_share_list: Arc<Mutex<HashMap<String, Vec<(IpAddr, bool)>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        assert_eq!(
            select_sharing_files_to_send(&my_files_to_share_list).is_empty(),
            true
        );
    }

    {
        let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
        let file_name = String::from("C:/docs/resume.pdf");

        my_files_to_share_list.lock().unwrap().insert(
            file_name.clone(),
            vec![
                (IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)), false),
                (IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12)), true),
                (IpAddr::V4(Ipv4Addr::new(91, 0, 4, 14)), false),
            ],
        );

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(file_name, vec![IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12))]);

        assert_eq!(
            select_sharing_files_to_send(&my_files_to_share_list),
            result
        );
    }

    {
        let file_1_name = String::from("D:/Games/Serious_sam.exe");
        let file_2_name = String::from("D:/free/film.mp4");
        let file_3_name = String::from("C:/resume.pdf");
        let file_4_name = String::from("crypt.txt");

        let my_files_to_share_list = Arc::new(Mutex::new(HashMap::new()));
        my_files_to_share_list.lock().unwrap().insert(
            file_1_name.clone(),
            vec![
                (IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)), true),
                (IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12)), true),
                (IpAddr::V4(Ipv4Addr::new(74, 24, 9, 6)), true),
                (IpAddr::V4(Ipv4Addr::new(64, 244, 92, 76)), true),
            ],
        );

        my_files_to_share_list.lock().unwrap().insert(
            file_2_name.clone(),
            vec![
                (IpAddr::V4(Ipv4Addr::new(74, 24, 9, 6)), false),
                (IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)), false),
            ],
        );

        my_files_to_share_list.lock().unwrap().insert(
            file_3_name.clone(),
            vec![
                (IpAddr::V4(Ipv4Addr::new(64, 244, 92, 76)), true),
                (IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)), false),
            ],
        );

        my_files_to_share_list
            .lock()
            .unwrap()
            .insert(file_4_name.clone(), vec![]);

        let mut result: HashMap<String, Vec<IpAddr>> = HashMap::new();
        result.insert(
            file_1_name,
            vec![
                IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1)),
                IpAddr::V4(Ipv4Addr::new(45, 67, 0, 12)),
                IpAddr::V4(Ipv4Addr::new(74, 24, 9, 6)),
                IpAddr::V4(Ipv4Addr::new(64, 244, 92, 76)),
            ],
        );

        result.insert(file_2_name, vec![]);
        result.insert(
            file_3_name,
            vec![IpAddr::V4(Ipv4Addr::new(64, 244, 92, 76))],
        );
        result.insert(file_4_name, vec![]);

        assert_eq!(
            select_sharing_files_to_send(&my_files_to_share_list),
            result
        );
    }
}

// !!! Hardware specific !!!

#[test]
fn filter_peers_test() {
    {
        let remote_addr = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let actual_peers = vec![remote_addr];
        let actual_result = filter_peers(&actual_peers);
        assert_eq!(actual_result.len(), 1);
        assert_eq!(actual_result, vec![remote_addr]);
    }

    {
        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 2));
        let actual_peers = vec![remote_addr_1, remote_addr_2];
        let actual_result = filter_peers(&actual_peers);
        assert_eq!(actual_result.len(), 2);
    }

    {
        let remote_addr_1 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 1));
        let remote_addr_2 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 2));
        let remote_addr_3 = IpAddr::V4(Ipv4Addr::new(231, 0, 0, 3));
        let actual_peers = vec![remote_addr_1, remote_addr_2, remote_addr_3];
        let actual_result = filter_peers(&actual_peers);
        assert_eq!(actual_result.len(), 2);
        assert_eq!(
            (actual_result == vec![remote_addr_1, remote_addr_2]
                || actual_result == vec![remote_addr_2, remote_addr_1]),
            true
        );
    }
}
