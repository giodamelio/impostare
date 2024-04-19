use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Config {
    databases: Vec<Database>,
    users: Vec<User>,
    database_permissions: Vec<DatabasePermission>,
    schema_permissions: Vec<SchemaPermission>,
    table_permissions: Vec<TablePermission>,
}

#[derive(Deserialize, Debug)]
struct Database {
    name: String,
    extensions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
    password_file: String,
}

#[derive(Deserialize, Debug)]
struct DatabasePermission {
    role: String,
    permissions: Vec<String>,
    databases: Vec<String>,
    make_default: bool,
}

#[derive(Deserialize, Debug)]
struct SchemaPermission {
    role: String,
    permissions: Vec<String>,
    schemas: Vec<String>,
    make_default: bool,
}

#[derive(Deserialize, Debug)]
struct TablePermission {
    role: String,
    permissions: Vec<String>,
    tables: Tables,
    make_default: bool,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Tables {
    #[serde(deserialize_with = "all")]
    All,
    List(Vec<String>),
}

// Deserialize just the ALL half of the enum
// See: https://github.com/serde-rs/serde/issues/1158)
fn all<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "ALL")]
        Variant,
    }
    Helper::deserialize(deserializer).map(|_| ())
}
