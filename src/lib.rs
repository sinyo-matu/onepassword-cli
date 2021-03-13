mod error;
mod output;
mod tests;

use chrono::{prelude::*, Duration};
use error::Error;
use serde::de::DeserializeOwned;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct OpCLI {
    expiration_time: DateTime<Utc>,
    session: String,
    keep_session_alive: bool,
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
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(pass.as_bytes()).await?;
        let output = child.wait_with_output().await?;
        handle_op_signin_error(String::from_utf8_lossy(&output.stderr).to_string()).await?;
        let expiration_time = Utc::now() + Duration::minutes(29);
        Ok(Self {
            expiration_time,
            session: String::from_utf8_lossy(&output.stdout).to_string(),
            keep_session_alive: alive,
        })
    }

    pub fn get(&self) -> GetCmd {
        GetCmd {
            cmd: "get".to_string(),
            session: self.session.to_string(),
        }
    }

    pub fn create(&self) -> CreateCmd {
        CreateCmd {
            cmd: "create".to_string(),
            session: self.session.to_string(),
        }
    }
}

pub trait FirstCmd {
    fn cmd(&self) -> String;

    fn session(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct GetCmd {
    cmd: String,
    session: String,
}

impl FirstCmd for GetCmd {
    fn cmd(&self) -> String {
        self.cmd.clone()
    }

    fn session(&self) -> String {
        self.session.clone()
    }
}

macro_rules! its_first_cmd {
    ($i:ident) => {
        #[derive(Debug, Clone)]
        pub struct $i {
            cmd: String,
            session: String,
        }

        impl FirstCmd for $i {
            fn cmd(&self) -> String {
                self.cmd.clone()
            }

            fn session(&self) -> String {
                self.session.clone()
            }
        }
    };
}

its_first_cmd!(CreateCmd);

impl GetCmd {
    pub fn account(&self) -> AccountCmd {
        let flags: Vec<String> = Vec::new();
        AccountCmd {
            first: self.clone(),
            cmd: "account".to_string(),
            flags,
        }
    }

    pub fn item(&self, item_name: &str) -> ItemLiteCmd {
        let flags: Vec<String> = vec![
            item_name.to_string(),
            "--fields".to_string(),
            "website,username,password".to_string(),
        ];
        ItemLiteCmd {
            first: self.clone(),
            cmd: "item".to_string(),
            flags,
        }
    }

    pub fn document(&self, document_name: &str) -> GetDocumentCmd {
        let flags: Vec<String> = vec![document_name.to_string()];
        GetDocumentCmd {
            first: self.clone(),
            cmd: "document".to_string(),
            flags,
        }
    }

    pub fn totp(&self, item_name: &str) -> GetTotpCmd {
        let flags: Vec<String> = vec![item_name.to_string()];

        GetTotpCmd {
            first: self.clone(),
            cmd: "totp".to_string(),
            flags,
        }
    }
}
impl CreateCmd {
    pub fn document(&self, path: &str) -> CreateDocumentCmd {
        let flags: Vec<String> = vec![path.to_string()];
        CreateDocumentCmd {
            first: self.clone(),
            cmd: "document".to_string(),
            flags,
        }
    }
}

#[async_trait::async_trait]
pub trait SecondCmd {
    type Output: DeserializeOwned;
    type First: FirstCmd + Clone;

    fn first(&self) -> Self::First;

    fn cmd(&self) -> String;

    fn flags(&self) -> Vec<String>;
}

#[async_trait::async_trait]
pub trait SecondCmdExt: SecondCmd {
    fn add_flag(&mut self, flags: &[&str]) -> &Self {
        for flag in flags {
            if !self.flags().contains(&flag.to_string()) {
                self.flags().push(flag.to_string())
            }
        }
        self
    }

    async fn run(&self) -> Result<Self::Output> {
        let mut args: Vec<String> = vec![
            self.first().cmd(),
            self.cmd(),
            "--session".to_string(),
            self.first().session().trim().to_string(),
        ];
        if !self.flags().is_empty() {
            self.flags().into_iter().for_each(|flag| args.push(flag))
        }
        let out_str: &str = &exec_command(args).await?;
        Ok(serde_json::from_str(out_str)?)
    }
}

impl<H: SecondCmd> SecondCmdExt for H {}

#[derive(Debug)]
pub struct AccountCmd {
    first: GetCmd,
    cmd: String,
    flags: Vec<String>,
}

#[async_trait::async_trait]
impl SecondCmd for AccountCmd {
    type Output = output::Account;
    type First = GetCmd;
    fn first(&self) -> GetCmd {
        self.first.clone()
    }

    fn cmd(&self) -> String {
        self.cmd.clone()
    }

    fn flags(&self) -> Vec<String> {
        self.flags.clone()
    }
}

macro_rules! its_second_cmd {
    ($f:ident,$i:ident,$e:ident) => {
        #[derive(Debug)]
        pub struct $i {
            first: $f,
            cmd: String,
            flags: Vec<String>,
        }

        #[async_trait::async_trait]
        impl SecondCmd for $i {
            type Output = output::$e;
            type First = $f;
            fn first(&self) -> $f {
                self.first.clone()
            }

            fn cmd(&self) -> String {
                self.cmd.clone()
            }

            fn flags(&self) -> Vec<String> {
                self.flags.clone()
            }
        }
    };
}

its_second_cmd!(GetCmd, ItemLiteCmd, ItemLite);
its_second_cmd!(GetCmd, GetDocumentCmd, Value);
its_second_cmd!(GetCmd, GetTotpCmd, Value);
its_second_cmd!(CreateCmd, CreateDocumentCmd, CreateDocument);

#[inline]
async fn exec_command(args: Vec<String>) -> Result<String> {
    let child = Command::new("op")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output().await?;
    handle_op_exec_error(String::from_utf8_lossy(&output.stderr).to_string()).await?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[inline]
async fn handle_op_signin_error(std_err: String) -> std::result::Result<(), Error> {
    if std_err.contains("401") {
        return Err(Error::OPSignInError("wrong password".to_string()));
    }
    Ok(())
}

#[inline]
async fn handle_op_exec_error(std_err: String) -> std::result::Result<(), Error> {
    if std_err.contains("doesn't seem to be an item") {
        return Err(Error::ItemQueryError("Item not founded".to_string()));
    } else if std_err.trim().contains("Invalid session token") {
        return Err(Error::ItemQueryError("In valid session token".to_string()));
    } else if std_err.trim().contains("More than one item matches") {
        return Err(Error::ItemQueryError(
            "More than one item matches".to_string(),
        ));
    }
    Ok(())
}
