use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub databases: Vec<Database>,
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub name: String,
    pub extensions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub name: String,
    pub password_file: String,
    pub permissions: Option<HashMap<String, Permissions>>,
}

pub type Permissions = Vec<String>;
