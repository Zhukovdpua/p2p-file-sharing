#[macro_use]
extern crate clap;
use clap::{Arg, App, SubCommand};
use p2p_file_sharing::lib::CommandType;

fn main() {

    //should we use multiple for args ?
    let matches = App::new("P2P File Sharing")
        .author(crate_authors!())
        .version(crate_version!())
        .usage("[share <filepath>] - Add to the list of files that are available for download from this client\n    \
                [scan] - Refresh the list of peers and files that are available for download\n    \
                [ls] - Show the list of files that this client can download\n    \
                [download <file_name> -o <save_path>] - Download the file with the specified name and save along the specified path\n    \
                [status] - Show a list of current downloads and a list of files that are available for download")
        .about("The application allows for decentralized file sharing")
        .arg(Arg::with_name("ARG1").required(true).help("Command name"))
        .arg(Arg::with_name("ARG2").help("FILE_PATH for share command or FILE_NAME for download command"))
        .arg(Arg::with_name("ARG3").help("Command flag"))
        .arg(Arg::with_name("ARG4").help("SAVE PATH"))
        .get_matches();

        if matches.is_present("ARG1"){
            println!("ARG1 was used!");
            if let Some(arg1_val) = matches.value_of("ARG1"){

                //let m_val = arg1_val;
                if arg1_val == "share"{

                   ///////////
                }
                else if arg1_val == "scan"{

                    ///////////
                }
                else if arg1_val == "ls"{

                    ////////////
                }
                else if arg1_val == "download" {
                    if matches.is_present("ARG2"){
                        if matches.is_present("ARG3"){
                            if matches.is_present("ARG4"){

                                if let Some(arg2_val) = matches.value_of("ARG2"){

                                    if let Some(arg3_val) = matches.value_of("ARG3"){

                                        if let Some(arg4_val) = matches.value_of("ARG4"){

                                            ////////////////////////////////////
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                else if arg1_val == "status" {

                    ///////////
                }
            }
        }
}
