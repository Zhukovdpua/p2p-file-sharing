//use serde::{Serialize, Deserialize};

pub mod lib{

 //   #[derive(Serialize, Deserialize, Debug)]
  pub enum CommandType{
        Share(String),
        Scan,
        Ls,
        Download(String, String),
        Status
    }
}
