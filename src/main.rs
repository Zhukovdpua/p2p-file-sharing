#[macro_use]
extern crate clap;
mod functions;
mod commands;

use clap::{Arg, App};


use commands::{share, scan, ls, download_with_path,
               download_without_path, status};

fn main() {

    let matches = App::new("P2P File Sharing")
        .author(crate_authors!())
        .version(crate_version!())
        .usage("Command name must be written in lower case!\n    \
                [share <filepath>] - Add to the list of files that are available for download from this client\n    \
                [scan] - Refresh the list of peers and files that are available for download\n    \
                [ls] - Show the list of files that this client can download\n    \
                [download <file_name> -o <save_path>] - Download the file with the specified name and save along the specified path\n    \
                [status] - Show a list of current downloads and a list of files that are available for download")
        .about("The application allows for decentralized file sharing.")
        .arg(Arg::with_name("ARG1").required(true).help("Command name"))
        .arg(Arg::with_name("ARG2").help("FILE_PATH for the [share] command or FILE_NAME for the [download] command"))
        .arg(Arg::with_name("ARG3").help("SAVE PATH"))
        .arg(Arg::with_name("FLG1").help("Flag for saving with a forward string name").short("o"))
        .get_matches();

        if matches.is_present("ARG1"){

            if let Some(arg1_val) = matches.value_of("ARG1"){

                match arg1_val {

                    "share" => {
                        if let Some(arg2_val) = matches.value_of("ARG2") {
                            share(arg2_val);
                        }
                        else{

                            println!("Error with filepath!");
                        }
                    }

                    "scan" => {

                        scan();

                    }

                    "ls" => {

                        ls();
                    }

                    "download" => {

                        if matches.is_present("ARG2"){
                            if let Some(arg2_val) = matches.value_of("ARG2"){

                                    if matches.is_present("FLG1") {
                                        if matches.is_present("ARG3") {

                                            if let Some(arg3_val) = matches.value_of("ARG3") {

                                                download_with_path(arg2_val, arg3_val);
                                            }
                                        }
                                    }
                                    else {

                                        download_without_path(arg2_val);
                                    }
                            }
                        }
                    }

                    "status" => {

                        status();
                    }

                    _ => println!("Wrong command name! Try -h for more info.")
                }
            }
        }
}
