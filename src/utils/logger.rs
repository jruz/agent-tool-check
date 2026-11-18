mod env;

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::PathBuf;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to create log dir: {0}")]
    CreateDir(std::io::Error),
    #[error("{0}")]
    Env(#[from] env::Error),
    #[error("failed to open log file: {0}")]
    OpenFile(std::io::Error),
    #[error("failed to serialize log entry: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to write log entry: {0}")]
    Write(std::io::Error),
}

#[derive(Serialize)]
struct LogEntry {
    #[serde(rename = "Attributes")]
    attributes: HashMap<String, String>,
    #[serde(rename = "Body")]
    body: String,
    #[serde(rename = "SeverityText")]
    severity_text: &'static str,
    #[serde(rename = "Timestamp")]
    timestamp: String,
}

fn log_dir() -> Result<PathBuf, Error> {
    let state_dir = env::state_home()?;
    let path = state_dir.join("agent-tool-check");
    Ok(path)
}

fn write_entry(name: &str, entry: &impl Serialize) -> Result<(), Error> {
    let dir = log_dir()?;
    fs::create_dir_all(&dir).map_err(Error::CreateDir)?;

    let line = serde_json::to_string(entry).map_err(Error::Serialize)?;

    let path = dir.join(name);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)
        .map_err(Error::OpenFile)?;

    let record = format!("{line}\n");
    let bytes = record.as_bytes();
    file.write_all(bytes).map_err(Error::Write)?;
    Ok(())
}

pub fn error(body: &str, attributes: HashMap<String, String>) -> Result<(), Error> {
    let entry = LogEntry {
        attributes,
        body: body.to_owned(),
        severity_text: "ERROR",
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    write_entry("errors.log", &entry)
}

pub fn denial(entry: &serde_json::Value) -> Result<(), Error> {
    write_entry("denial.log", entry)
}
