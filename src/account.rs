use crate::db::DB;

#[derive(Clone, Debug)]
pub struct Account {
    pub email: String,
    pub password: String,
}

impl Account {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }

    pub fn save(&self) -> sled::Result<()> {
        DB.insert("account:email", self.email.as_bytes())?;
        DB.insert("account:password", self.password.as_bytes())?;
        DB.flush()?;
        Ok(())
    }

    pub fn load() -> sled::Result<Option<Self>> {
        let email = DB
            .get("account:email")?
            .and_then(|v| String::from_utf8(v.to_vec()).ok());
        let password = DB
            .get("account:password")?
            .and_then(|v| String::from_utf8(v.to_vec()).ok());

        if let (Some(email), Some(password)) = (email, password) {
            Ok(Some(Self { email, password }))
        } else {
            Ok(None)
        }
    }
}
