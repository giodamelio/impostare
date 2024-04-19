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
    statements.extend(create_databases(&config)?);

    println!("Statements: {:?}", statements);

    Ok(())
}

fn create_databases(config: &Config) -> Result<Vec<Statement>> {
    Ok(config
        .databases
        .iter()
        .map(|db| {
            (
                None,
                "CREATE DATABASE IF NOT EXISTS $1",
                vec![db.name.clone()],
            )
        })
        .collect())
}
