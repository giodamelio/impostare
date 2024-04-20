use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq)]
pub struct Statement {
    database: Option<String>,
    sql: String,
}

impl Statement {
    fn create<S: Into<Statement>>(statement: S) -> Self {
        statement.into()
    }
}

impl From<(String, String)> for Statement {
    fn from((db, sql): (String, String)) -> Self {
        Self {
            database: Some(db),
            sql,
        }
    }
}

impl From<(&str, &str)> for Statement {
    fn from((db, sql): (&str, &str)) -> Self {
        Self {
            database: Some(db.to_string()),
            sql: sql.to_string(),
        }
    }
}

impl From<String> for Statement {
    fn from(value: String) -> Self {
        Self {
            database: None,
            sql: value,
        }
    }
}

impl From<&str> for Statement {
    fn from(value: &str) -> Self {
        Self {
            database: None,
            sql: value.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    databases: Vec<Database>,
    extensions: Vec<Extension>,
    users: Vec<User>,
    database_permissions: Vec<DatabasePermission>,
    schema_permissions: Vec<SchemaPermission>,
    table_permissions: Vec<TablePermission>,
}

#[derive(Deserialize, Debug)]
struct Database {
    name: String,
}

impl ToSQLStatements for Database {
    fn to_sql_statements(self) -> Vec<Statement> {
        vec![format!("CREATE DATABASE {};", self.name).into()]
    }
}

#[derive(Deserialize, Debug)]
struct Extension {
    name: String,
    database: String,
}

impl ToSQLStatements for Extension {
    fn to_sql_statements(self) -> Vec<Statement> {
        vec![Statement {
            database: Some(self.database),
            sql: format!("CREATE EXTENSION IF NOT EXISTS {};", self.name),
        }]
    }
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
    password_file: String,
}

impl ToSQLStatements for User {
    fn to_sql_statements(self) -> Vec<Statement> {
        vec![format!("CREATE USER {};", self.name).into()]
    }
}

#[derive(Deserialize, Debug)]
struct DatabasePermission {
    role: String,
    permissions: Vec<String>,
    databases: Vec<String>,
}

impl ToSQLStatements for DatabasePermission {
    fn to_sql_statements(self) -> Vec<Statement> {
        let mut statements = vec![];

        for database in self.databases {
            statements.push(Statement {
                database: Some(database.clone()),
                sql: format!(
                    "GRANT {} ON DATABASE {} TO {};",
                    self.permissions.join(", "),
                    database,
                    self.role
                ),
            });
        }

        statements
    }
}

#[derive(Deserialize, Debug)]
struct SchemaPermission {
    role: String,
    permissions: Vec<String>,
    database: String,
    schemas: Vec<String>,
    make_default: bool,
}

impl ToSQLStatements for SchemaPermission {
    fn to_sql_statements(self) -> Vec<Statement> {
        let mut statements = vec![];

        for schema in self.schemas {
            statements.push(Statement {
                database: Some(self.database.clone()),
                sql: format!(
                    "GRANT {} ON SCHEMA {} TO {};",
                    self.permissions.join(", "),
                    schema,
                    self.role
                ),
            });

            if self.make_default {
                statements.push(Statement {
                    database: Some(self.database.clone()),
                    sql: format!(
                        "ALTER DEFAULT PRIVILEGES GRANT {} ON SCHEMAS TO {};",
                        self.permissions.join(", "),
                        self.role
                    ),
                });
            }
        }

        statements
    }
}

#[derive(Deserialize, Debug)]
struct TablePermission {
    role: String,
    permissions: Vec<String>,
    database: String,
    tables: Tables,
    make_default: bool,
}

impl ToSQLStatements for TablePermission {
    fn to_sql_statements(self) -> Vec<Statement> {
        let mut statements = vec![];

        let tables_string = match self.tables {
            // TODO: I know this shouldn't be hardcoded
            Tables::All => "ALL IN SCHEMA public".to_string(),
            Tables::List(tables) => format!("TABLE {}", tables.join(", ")),
        };

        statements.push(Statement {
            database: Some(self.database.clone()),
            sql: format!(
                "GRANT {} ON {} TO {};",
                self.permissions.join(", "),
                tables_string,
                self.role
            ),
        });

        if self.make_default {
            statements.push(Statement {
                database: Some(self.database.clone()),
                sql: format!(
                    "ALTER DEFAULT PRIVILEGES GRANT {} ON TABLES TO {};",
                    self.permissions.join(", "),
                    self.role
                ),
            });
        }

        statements
    }
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

trait ToSQLStatements {
    fn to_sql_statements(self) -> Vec<Statement>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_database() {
        let db = Database {
            name: "hello".to_string(),
        };
        assert_eq!(
            db.to_sql_statements(),
            vec![Statement::create("CREATE DATABASE hello;")],
        );
    }

    #[test]
    fn test_serialize_extension() {
        let ex = Extension {
            name: "timescaledb".to_string(),
            database: "metrics".to_string(),
        };
        assert_eq!(
            ex.to_sql_statements(),
            vec![Statement::create((
                "metrics",
                "CREATE EXTENSION IF NOT EXISTS timescaledb;"
            ))]
        );
    }

    #[test]
    fn test_serialize_user() {
        let user = User {
            name: "grafana".to_string(),
            password_file: "/a/file".to_string(),
        };
        assert_eq!(
            user.to_sql_statements(),
            vec![Statement::create("CREATE USER grafana;")]
        );
    }

    #[test]
    fn test_serialize_database_permission() {
        let dbp = DatabasePermission {
            role: "telegraf".to_string(),
            permissions: vec!["CONNECT".to_string()],
            databases: vec!["db1".to_string(), "db2".to_string()],
        };
        assert_eq!(
            dbp.to_sql_statements(),
            vec![
                Statement::create(("db1", "GRANT CONNECT ON DATABASE db1 TO telegraf;")),
                Statement::create(("db2", "GRANT CONNECT ON DATABASE db2 TO telegraf;")),
            ]
        );
    }

    #[test]
    fn test_serialize_schema_permission() {
        let sp = SchemaPermission {
            role: "telegraf".to_string(),
            permissions: vec!["CREATE".to_string()],
            database: "db1".to_string(),
            schemas: vec!["public".to_string(), "other".to_string()],
            make_default: false,
        };
        assert_eq!(
            sp.to_sql_statements(),
            vec![
                Statement::create(("db1", "GRANT CREATE ON SCHEMA public TO telegraf;")),
                Statement::create(("db1", "GRANT CREATE ON SCHEMA other TO telegraf;")),
            ]
        );

        let sp2 = SchemaPermission {
            role: "telegraf".to_string(),
            permissions: vec!["CREATE".to_string(), "USAGE".to_string()],
            database: "db1".to_string(),
            schemas: vec!["public".to_string(), "other".to_string()],
            make_default: true,
        };
        assert_eq!(
            sp2.to_sql_statements(),
            vec![
                Statement::create(("db1", "GRANT CREATE, USAGE ON SCHEMA public TO telegraf;")),
                Statement::create((
                    "db1",
                    "ALTER DEFAULT PRIVILEGES GRANT CREATE, USAGE ON SCHEMAS TO telegraf;"
                )),
                Statement::create(("db1", "GRANT CREATE, USAGE ON SCHEMA other TO telegraf;")),
                Statement::create((
                    "db1",
                    "ALTER DEFAULT PRIVILEGES GRANT CREATE, USAGE ON SCHEMAS TO telegraf;"
                )),
            ]
        );
    }

    #[test]
    fn test_serialize_table_permission() {
        let tp = TablePermission {
            role: "telegraf".to_string(),
            permissions: vec!["SELECT".to_string()],
            database: "db1".to_string(),
            tables: Tables::All,
            make_default: true,
        };
        assert_eq!(
            tp.to_sql_statements(),
            vec![
                Statement::create(("db1", "GRANT SELECT ON ALL IN SCHEMA public TO telegraf;")),
                Statement::create((
                    "db1",
                    "ALTER DEFAULT PRIVILEGES GRANT SELECT ON TABLES TO telegraf;"
                )),
            ]
        );

        let tp2 = TablePermission {
            role: "grafana".to_string(),
            permissions: vec!["SELECT".to_string(), "UPDATE".to_string()],
            database: "db1".to_string(),
            tables: Tables::List(vec!["table1".to_string(), "table2".to_string()]),
            make_default: true,
        };
        assert_eq!(
            tp2.to_sql_statements(),
            vec![
                Statement::create((
                    "db1",
                    "GRANT SELECT, UPDATE ON TABLE table1, table2 TO grafana;"
                )),
                Statement::create((
                    "db1",
                    "ALTER DEFAULT PRIVILEGES GRANT SELECT, UPDATE ON TABLES TO grafana;"
                )),
            ]
        );
    }
}
