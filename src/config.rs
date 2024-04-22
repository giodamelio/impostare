use anyhow::Result;
use postgres::error::SqlState;
use serde::{Deserialize, Deserializer};

use crate::statement::{Statement, Statements};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub databases: Vec<Database>,
    pub extensions: Vec<Extension>,
    pub users: Vec<User>,
    pub database_permissions: Vec<DatabasePermission>,
    pub schema_permissions: Vec<SchemaPermission>,
    pub table_permissions: Vec<TablePermission>,
}

impl ToSQLStatements for Config {
    fn to_sql_statements(&self) -> Statements {
        let mut statements = Statements::new();

        statements.extend(self.databases.iter().flat_map(|db| db.to_sql_statements()));
        statements.extend(self.extensions.iter().flat_map(|ex| ex.to_sql_statements()));
        statements.extend(self.users.iter().flat_map(|user| user.to_sql_statements()));

        statements
    }
}

#[derive(Deserialize, Debug)]
pub struct Database {
    name: String,
}

impl ToSQLStatements for Database {
    fn to_sql_statements(&self) -> Statements {
        vec![Statement {
            database: None,
            sql: format!("CREATE DATABASE {};", self.name),
            ignorable_errors: vec![SqlState::DUPLICATE_DATABASE],
        }]
        .into()
    }
}

#[derive(Deserialize, Debug)]
pub struct Extension {
    name: String,
    database: String,
}

impl ToSQLStatements for Extension {
    fn to_sql_statements(&self) -> Statements {
        vec![Statement {
            database: Some(self.database.clone()),
            sql: format!("CREATE EXTENSION IF NOT EXISTS {};", self.name),
            ignorable_errors: vec![],
        }]
        .into()
    }
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
    systemd_password_credential: Option<String>,
}

impl ToSQLStatements for User {
    fn to_sql_statements(&self) -> Statements {
        let mut statements = vec![];

        statements.push(Statement {
            database: None,
            sql: format!("CREATE USER {};", self.name),
            ignorable_errors: vec![SqlState::DUPLICATE_OBJECT],
        });

        if self.systemd_password_credential.is_some() {
            statements.push(Statement {
                database: None,
                sql: format!(
                    "ALTER USER {} WITH PASSWORD '{}';",
                    self.name, "super_secret_password"
                ),
                ignorable_errors: vec![],
            });
        }

        statements.into()
    }
}

#[derive(Deserialize, Debug)]
pub struct DatabasePermission {
    role: String,
    permissions: Vec<String>,
    databases: Vec<String>,
}

impl ToSQLStatements for DatabasePermission {
    fn to_sql_statements(&self) -> Statements {
        let mut statements = vec![];

        for database in self.databases.clone() {
            statements.push(Ok(Statement {
                database: Some(database.clone()),
                sql: format!(
                    "GRANT {} ON DATABASE {} TO {};",
                    self.permissions.join(", "),
                    database,
                    self.role
                ),
                ignorable_errors: vec![],
            }));
        }

        statements.into()
    }
}

#[derive(Deserialize, Debug)]
pub struct SchemaPermission {
    role: String,
    permissions: Vec<String>,
    database: String,
    schemas: Vec<String>,
    make_default: bool,
}

impl ToSQLStatements for SchemaPermission {
    fn to_sql_statements(&self) -> Statements {
        let mut statements = vec![];

        for schema in self.schemas.clone() {
            statements.push(Statement {
                database: Some(self.database.clone()),
                sql: format!(
                    "GRANT {} ON SCHEMA {} TO {};",
                    self.permissions.join(", "),
                    schema,
                    self.role
                ),
                ignorable_errors: vec![],
            });

            if self.make_default {
                statements.push(Statement {
                    database: Some(self.database.clone()),
                    sql: format!(
                        "ALTER DEFAULT PRIVILEGES GRANT {} ON SCHEMAS TO {};",
                        self.permissions.join(", "),
                        self.role
                    ),
                    ignorable_errors: vec![],
                });
            }
        }

        statements.into()
    }
}

#[derive(Deserialize, Debug)]
pub struct TablePermission {
    role: String,
    permissions: Vec<String>,
    database: String,
    tables: Tables,
    make_default: bool,
}

impl ToSQLStatements for TablePermission {
    fn to_sql_statements(&self) -> Statements {
        let mut statements = vec![];

        let tables_string = match self.tables.clone() {
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
            ignorable_errors: vec![],
        });

        if self.make_default {
            statements.push(Statement {
                database: Some(self.database.clone()),
                sql: format!(
                    "ALTER DEFAULT PRIVILEGES GRANT {} ON TABLES TO {};",
                    self.permissions.join(", "),
                    self.role
                ),
                ignorable_errors: vec![],
            });
        }

        statements.into()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Tables {
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

pub trait ToSQLStatements {
    fn to_sql_statements(&self) -> Statements;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_statement(
        database: Option<&'static str>,
        sql: &'static str,
        ignorable_errors: Vec<SqlState>,
    ) -> Statement {
        Statement {
            database: database.map(|name| name.to_string()),
            sql: sql.to_string(),
            ignorable_errors,
        }
    }

    fn has_statement(statement: Statement, statements: Statements) {
        let has_it = statements.iter().any(|st| match st {
            Err(_) => false,
            Ok(s) => *s == statement,
        });

        assert!(
            has_it,
            "{:#?} does not contain {:#?}",
            statements, statement
        )
    }

    #[test]
    fn test_is_ignoreable_error() {
        let statement = Statement {
            database: None,
            sql: "".into(),
            ignorable_errors: vec![SqlState::DUPLICATE_DATABASE],
        };

        // I know this is a little nasty,
        // there is no other way to create a postgres::Error publicly(ish)
        let error = postgres::Error::__private_api_timeout();
        assert!(!statement.is_ignorable_error(&error));
    }

    #[test]
    fn test_serialize_database() -> Result<()> {
        let db = Database {
            name: "hello".to_string(),
        };

        has_statement(
            create_statement(
                None,
                "CREATE DATABASE hello;",
                vec![SqlState::DUPLICATE_DATABASE],
            ),
            db.to_sql_statements(),
        );

        Ok(())
    }

    #[test]
    fn test_serialize_extension() {
        let ex = Extension {
            name: "timescaledb".to_string(),
            database: "metrics".to_string(),
        };

        has_statement(
            create_statement(
                Some("metrics"),
                "CREATE EXTENSION IF NOT EXISTS timescaledb;",
                vec![],
            ),
            ex.to_sql_statements(),
        );
    }

    #[test]
    fn test_serialize_user() {
        let user = User {
            name: "grafana".to_string(),
            systemd_password_credential: Some("whisper-whisper".to_string()),
        };
        has_statement(
            create_statement(
                None,
                "CREATE USER grafana;",
                vec![SqlState::DUPLICATE_OBJECT],
            ),
            user.to_sql_statements(),
        );
    }

    #[test]
    fn test_serialize_database_permission() {
        let dbp = DatabasePermission {
            role: "telegraf".to_string(),
            permissions: vec!["CONNECT".to_string()],
            databases: vec!["db1".to_string(), "db2".to_string()],
        };
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT CONNECT ON DATABASE db1 TO telegraf;",
                vec![],
            ),
            dbp.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db2"),
                "GRANT CONNECT ON DATABASE db2 TO telegraf;",
                vec![],
            ),
            dbp.to_sql_statements(),
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
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT CREATE ON SCHEMA public TO telegraf;",
                vec![],
            ),
            sp.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT CREATE ON SCHEMA other TO telegraf;",
                vec![],
            ),
            sp.to_sql_statements(),
        );

        let sp2 = SchemaPermission {
            role: "telegraf".to_string(),
            permissions: vec!["CREATE".to_string(), "USAGE".to_string()],
            database: "db1".to_string(),
            schemas: vec!["public".to_string(), "other".to_string()],
            make_default: true,
        };
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT CREATE, USAGE ON SCHEMA public TO telegraf;",
                vec![],
            ),
            sp2.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "ALTER DEFAULT PRIVILEGES GRANT CREATE, USAGE ON SCHEMAS TO telegraf;",
                vec![],
            ),
            sp2.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT CREATE, USAGE ON SCHEMA other TO telegraf;",
                vec![],
            ),
            sp2.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "ALTER DEFAULT PRIVILEGES GRANT CREATE, USAGE ON SCHEMAS TO telegraf;",
                vec![],
            ),
            sp2.to_sql_statements(),
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
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT SELECT ON ALL IN SCHEMA public TO telegraf;",
                vec![],
            ),
            tp.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "ALTER DEFAULT PRIVILEGES GRANT SELECT ON TABLES TO telegraf;",
                vec![],
            ),
            tp.to_sql_statements(),
        );

        let tp2 = TablePermission {
            role: "grafana".to_string(),
            permissions: vec!["SELECT".to_string(), "UPDATE".to_string()],
            database: "db1".to_string(),
            tables: Tables::List(vec!["table1".to_string(), "table2".to_string()]),
            make_default: true,
        };
        has_statement(
            create_statement(
                Some("db1"),
                "GRANT SELECT, UPDATE ON TABLE table1, table2 TO grafana;",
                vec![],
            ),
            tp2.to_sql_statements(),
        );
        has_statement(
            create_statement(
                Some("db1"),
                "ALTER DEFAULT PRIVILEGES GRANT SELECT, UPDATE ON TABLES TO grafana;",
                vec![],
            ),
            tp2.to_sql_statements(),
        );
    }
}
