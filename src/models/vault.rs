use serde::{Serialize, Deserialize};

use crate::utils::crypto::ArgonKey;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Item {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Vault {
    pub nonce: i32,
    pub items: Vec<Item>,
    pub key: Option<ArgonKey>,
}

impl Vault {
    pub(crate) fn new() -> Self {
        Self { 
            nonce: 1,
            items: vec![
                Item {
                    id: 0,
                    name: "New Item".into(),
                    username: String::new(),
                    password: String::new(),
                    url: String::new(),
                    notes: String::new(),
                },
            ],
            key: None,
        }
    }
}