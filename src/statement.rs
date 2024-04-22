use std::ops::{Deref, DerefMut};

use anyhow::Result;
use postgres::error::SqlState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub database: Option<String>,
    pub sql: String,
    pub ignorable_errors: Vec<SqlState>,
}

impl Statement {
    pub fn is_ignorable_error(&self, err: &postgres::Error) -> bool {
        match err.code() {
            None => false,
            Some(code) => self.ignorable_errors.iter().any(|c| c == code),
        }
    }
}

#[derive(Debug)]
pub struct Statements(Vec<Result<Statement>>);

impl Statements {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl Deref for Statements {
    type Target = Vec<Result<Statement>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Statements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Statements {
    type Item = Result<Statement>;
    type IntoIter = std::vec::IntoIter<Result<Statement>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Result<Statement>> for Statements {
    fn from_iter<T: IntoIterator<Item = Result<Statement>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Vec<Statement>> for Statements {
    fn from(value: Vec<Statement>) -> Self {
        Self(value.iter().map(|s| Ok(s.clone())).collect())
    }
}

impl From<Vec<Result<Statement>>> for Statements {
    fn from(value: Vec<Result<Statement>>) -> Self {
        Self(value)
    }
}
