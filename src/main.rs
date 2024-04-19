mod config;

use std::fs;

use anyhow::Result;

use config::Config;

type Statement = (Option<String>, &'static str, Vec<String>);

fn main() -> Result<()> {
    let toml_content = fs::read_to_string("db.toml")?;
    let config: Config = toml::from_str(&toml_content)?;

    println!("{:#?}", config);

    let mut statements = vec![];
    statements.extend(create_databases(&config));
    statements.extend(load_extensions(&config));

    println!("Statements: {:#?}", statements);

    Ok(())
}

fn create_databases(config: &Config) -> Vec<Statement> {
    config
        .databases
        .iter()
        .map(|db| {
            (
                None,
                "CREATE DATABASE IF NOT EXISTS $1",
                vec![db.name.clone()],
            )
        })
        .collect()
}

fn load_extensions(config: &Config) -> Vec<Statement> {
    config
        .databases
        .iter()
        .flat_map(|db| {
            db.extensions.iter().map(|ex| {
                (
                    Some(db.name.clone()),
                    "CREATE EXTENSION IF NOT EXISTS $1",
                    vec![ex.clone()],
                )
            })
        })
        .collect()
}
