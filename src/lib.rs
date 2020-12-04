#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    use crate::OpCLI;

    #[test]
    fn test_new_with_pass() -> Result<()> {
        let op_cli = block_on(OpCLI::new_with_pass(
            "my".to_string(),
            "64659027Qy".to_string(),
            false,
        ))?;
        assert_eq!(op_cli.session.len(), 44);
        Ok(())
    }
    #[test]
    fn test_new_with_pass_input() -> Result<()> {
        let op_cli = block_on(OpCLI::new_with_pass_input("my".to_string(), false))?;
        assert_eq!(op_cli.session.len(), 44);
        Ok(())
    }

    #[test]
    fn test_get_username_password() -> Result<()> {
        let op_cli = block_on(OpCLI::new_with_pass(
            "my".to_string(),
            "64659027Qy".to_string(),
            false,
        ))?;
        let item = block_on(op_cli.get_username_password("SBI証券"))?;
        assert_eq!(item.username, "seelerei0130".to_string());
        Ok(())
    }
}

use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("signIn error : {0}")]
    SignInError(#[from] std::io::Error),
    #[error("query item error : {0}")]
    QueryItemError(std::io::Error),
    #[error("deserialize error : {0}")]
    QueryItemDeserializeError(#[from] serde_json::error::Error),
}

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
            .stdout(Stdio::piped())
            .spawn()?;
        {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(pass.as_bytes())?
        }
        let output = child.wait_with_output()?;
        let expiration_time = Utc::now() + Duration::minutes(29);
        Ok(Self {
            expiration_time: expiration_time,
            session: String::from_utf8_lossy(&output.stdout).to_string(),
            keep_session_alive: alive,
        })
    }

    pub async fn new_with_pass_input(user: String, alive: bool) -> Result<Self> {
        let mut child = Command::new("op")
            .arg("signin")
            .arg(user)
            .arg("--raw")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        {
            let stdin = child.stdin.as_mut().unwrap();
            print!("Please entry your 1password master password:");
            let mut pass = String::new();
            io::stdin().read_line(&mut pass)?;
            stdin.write_all(pass.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        let expiration_time = Utc::now() + Duration::minutes(29);
        Ok(Self {
            expiration_time: expiration_time,
            session: String::from_utf8_lossy(&output.stdout).to_string(),
            keep_session_alive: alive,
        })
    }

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
        Ok(item_lite)
    }
}

async fn exec_command(args: &[&str]) -> Result<String> {
    let child = Command::new("op")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    // println!("2{:?}", &output.stdout);
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
