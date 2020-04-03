//! Crate for containing common commands for client and daemon
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CommandType {
    Share(String),
    Scan,
    Ls,
    Download(String, String),
    Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseType {
    ShareScan,
    LsStatus(String),
    Download(bool),
    Error(String)
}
