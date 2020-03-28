use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
 pub enum CommandType{
    Share(String),
    Scan,
    Ls,
    Download(String, String),
    Status
}

