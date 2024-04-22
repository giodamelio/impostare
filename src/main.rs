mod config;
mod statement;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use log::{debug, error, info, trace};
use postgres::{Client, Config as PgConfig, NoTls};

use crate::config::{Config, ToSQLStatements};
use crate::statement::{Statement, Statements};

struct DB {
    connections: HashMap<Option<String>, Client>,
    base_config: PgConfig,
    dry_run: bool,
}

impl DB {
    fn connect(params: &str, dry_run: bool) -> Result<Self> {
        Ok(Self {
            connections: HashMap::new(),
            base_config: params.parse()?,
            dry_run,
        })
    }

    // Get a connection if it exists, otherwise create it first
    fn connection(&mut self, dbname: Option<&String>) -> Result<&mut Client> {
        Ok(match self.connections.entry(dbname.cloned()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let mut config = self.base_config.clone();
                if let Some(name) = dbname {
                    config.dbname(name);
                }
                debug!("Creating connection to database: {:?}", dbname);
                let connection = config.connect(NoTls)?;
                e.insert(connection)
            }
        })
    }

    fn execute(
        &mut self,
        statement: &Statement,
    ) -> Result<std::result::Result<u64, postgres::Error>> {
        trace!(
            "Executing SQL statement (database: {:?}): {:?}",
            statement.database.clone().unwrap_or("None".to_string()),
            statement.sql,
        );
        let conn = self.connection(statement.database.as_ref())?;
        Ok(conn.execute(&statement.sql, &[]))
    }
}

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;

    let toml_content = fs::read_to_string("db.toml")?;
    let config: Config = toml::from_str(&toml_content)?;
    trace!("Full config: {:#?}", config);

    let mut db = DB::connect(
        "host=/home/giodamelio/projects/impostare/.devenv/run/postgres user=postgres",
        false,
    )?;

    let mut statements = Statements::new();

    statements.extend(create_databases(&config));
    statements.extend(load_extensions(&config));
    statements.extend(create_users(&config));

    info!("Executing {} statments", statements.len());

    for statement in statements {
        match statement {
            Err(err) => error!("Could not build statement: {}", err),
            Ok(statement) => match db.execute(&statement)? {
                Err(err) => {
                    if !statement.is_ignorable_error(&err) {
                        error!("Statement failed: {}", statement.sql);
                    }
                }
                Ok(_) => info!("Statement succeded: {}", statement.sql),
            },
        };
    }

    Ok(())
}

fn create_databases(config: &Config) -> Statements {
    config
        .databases
        .iter()
        .flat_map(|db| db.to_sql_statements())
        .collect()
}

fn load_extensions(config: &Config) -> Statements {
    config
        .extensions
        .iter()
        .flat_map(|ex| ex.to_sql_statements())
        .collect()
}

fn create_users(config: &Config) -> Statements {
    config
        .users
        .iter()
        .flat_map(|user| user.to_sql_statements())
        .collect()
}
