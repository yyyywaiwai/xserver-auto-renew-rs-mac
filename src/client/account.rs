use bincode::{Decode, Encode};

#[derive(Clone, Debug, Encode, Decode)]
pub struct Account {
    pub email: String,
    pub password: String,
}
