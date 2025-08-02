use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Item {
    id: i32,
    name: String,
    username: String,
    password: String,
    url: String,
    notes: String,
}

#[derive(Clone)]
pub(crate) struct ItemSlint {
    id: i32,
    name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Vault {
    nonce: i32,
    items: Vec<Item>,
}

impl Vault {
    pub(crate) fn new() -> Self {
        Self { nonce: 1, items: vec![
            Item {
                id: 0,
                name: "New Item".into(),
                username: "".into(),
                password: "".into(),
                url: "".into(),
                notes: "".into(),
            }
        ]}
    }
}
