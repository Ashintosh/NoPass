use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::utils::{crypto::ArgonKey, zerobyte::ZeroByte};



#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub(crate) struct Item {
    pub id: i32,
    pub name: ZeroByte,
    pub username: ZeroByte,
    pub password: ZeroByte,
    pub url: ZeroByte,
    pub notes: ZeroByte,
}

#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
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
                    username: ZeroByte::with_capacity(0),
                    password: ZeroByte::with_capacity(0),
                    url: ZeroByte::with_capacity(0),
                    notes: ZeroByte::with_capacity(0),
                },
            ],
            key: None,
        }
    }
}