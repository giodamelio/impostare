mod config;
mod statement;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use argh::FromArgs;
use log::{debug, error, info, trace};
use postgres::{Client, Config as PgConfig, NoTls};

use crate::config::{Config, ToSQLStatements};
use crate::statement::Statement;

struct DB {
    connections: HashMap<Option<String>, Client>,
    base_config: PgConfig,
    dry_run: bool,
}

impl DB {
    fn connect(params: String, dry_run: bool) -> Result<Self> {
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
            "Executing SQL statement (database: {:?}): {}",
            statement.database.clone().unwrap_or("None".to_string()),
            statement,
        );
        let conn = self.connection(statement.database.as_ref())?;
        Ok(conn.execute(&statement.sql, &[]))
    }
}

#[derive(Debug, FromArgs)]
/// Simple CLI to allow setting up PostgreSQL Databases, Users and Permissions declaratively
struct Args {
    #[argh(positional)]
    /// file with PostgreSQL connection string
    connection_string_file: std::path::PathBuf,

    #[argh(positional)]
    /// file the impostare config
    config_file: std::path::PathBuf,
}

fn main() -> Result<()> {
    // Setup the logging
    pretty_env_logger::formatted_builder()
        .filter(Some("impostare"), log::LevelFilter::Info)
        .init();

    // Parse the CLI args
    let args: Args = argh::from_env();
    info!("CLI Args: {:#?}", args);

    // Read the config
    let toml_content = fs::read_to_string(args.config_file)?;
    let config: Config = toml::from_str(&toml_content)?;
    trace!("Full config: {:#?}", config);

    // Setup the DB connection multiplexer
    let mut db = DB::connect(std::fs::read_to_string(args.connection_string_file)?, false)?;

    let statements = config.to_sql_statements();

    info!("Executing {} statments", statements.len());

    for statement in statements {
        match statement {
            Err(err) => error!("Could not build statement: {}", err),
            Ok(statement) => match db.execute(&statement)? {
                Err(err) => {
                    if !statement.is_ignorable_error(&err) {
                        error!("Statement failed: {}", statement);
                    }
                }
                Ok(_) => info!("Statement succeded: {}", statement),
            },
        };
    }

    Ok(())
}
