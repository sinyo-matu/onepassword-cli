// mod tests;

use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{self, AsyncWriteExt};
use tokio::process::Command;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("signIn system error : {0}")]
    IOError(#[from] std::io::Error),
    #[error("signIn error : {0}")]
    SignInError(String),
    #[error("query item error : {0}")]
    QueryItemError(String),
    #[error("deserialize error : {0}")]
    QueryItemDeserializeError(#[from] serde_json::error::Error),
}

#[derive(Clone)]
pub struct OpCLI {
    expiration_time: DateTime<Utc>,
    session: String,
    keep_session_alive: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemLite {
    pub website: String,
    pub username: String,
    pub password: String,
}

impl OpCLI {
    pub async fn new_with_pass(user: String, pass: String, alive: bool) -> Result<Self> {
        let mut child = Command::new("op")
            .arg("signin")
            .arg(user)
            .arg("--raw")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(pass.as_bytes()).await?
        }
        let output = child.wait_with_output().await?;
        if String::from_utf8_lossy(&output.stderr)
            .to_string()
            .contains("Authentication")
        {
            return Err(Error::QueryItemError(String::from("Unauthorized")));
        }
        let expiration_time = Utc::now() + Duration::minutes(29);
        io::stdout().write_all(b"signIn successfully\n").await?;
        Ok(Self {
            expiration_time: expiration_time,
            session: String::from_utf8_lossy(&output.stdout).to_string(),
            keep_session_alive: alive,
        })
    }

    // pub async fn new_with_pass_input(user: String, alive: bool) -> Result<Self> {
    //     let mut child = Command::new("op")
    //         .arg("signin")
    //         .arg(user)
    //         .arg("--raw")
    //         .stdin(Stdio::piped())
    //         .stdout(Stdio::piped())
    //         .spawn()?;
    //     {
    //         let stdin = child.stdin.as_mut().unwrap();
    //         let mut pass = Vec::new();
    //     }
    //     let output = child.wait_with_output().await?;
    //     let expiration_time = Utc::now() + Duration::minutes(29);
    //     Ok(Self {
    //         expiration_time: expiration_time,
    //         session: String::from_utf8_lossy(&output.stdout).to_string(),
    //         keep_session_alive: alive,
    //     })
    // }

    pub async fn get_username_password(&self, item_name: &str) -> Result<ItemLite> {
        let output = exec_command(&[
            "get",
            "item",
            item_name,
            "--session",
            &self.session.trim(),
            "--fields",
            "website,username,password",
        ])
        .await?;
        let item_lite: ItemLite = serde_json::from_str(&output)?;
        io::stdout()
            .write_all(format!("Got {}\n", item_name).as_bytes())
            .await?;
        Ok(item_lite)
    }
}

async fn exec_command(args: &[&str]) -> Result<String> {
    let child = Command::new("op")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    if String::from_utf8_lossy(&output.stderr)
        .to_string()
        .contains("doesn't seem to be an item")
    {
        return Err(Error::QueryItemError(String::from("Wrong Item name")));
    }
    // println!("2{:?}", &output.stdout);
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
