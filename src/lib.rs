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

//OpCLI have expiration_time field what is the token's expiration time.
//Intent to implement some method to auto renew the token. //TODO
#[derive(Clone)]
pub struct OpCLI {
    expiration_time: DateTime<Utc>,
    session: String,
}

impl OpCLI {
    pub async fn new_with_pass(username: &str, password: &str) -> Result<Self> {
        let mut child = Command::new("op")
            .arg("signin")
            .arg(username)
            .arg("--raw")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(password.as_bytes()).await?;
        let output = child.wait_with_output().await?;
        handle_op_signin_error(String::from_utf8_lossy(&output.stderr).to_string()).await?;
        let expiration_time = Utc::now() + Duration::minutes(29);
        Ok(Self {
            expiration_time,
            session: String::from_utf8_lossy(&output.stdout).to_string(),
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

    pub fn list(&self) -> ListCmd {
        ListCmd {
            cmd: "list".to_string(),
            session: self.session.to_string(),
        }
    }
}

//I need this trait because because there are many different first cmds but they
//all need same methods to return their fields, we need them later.
//The Sealed Pattern can prevent others impl this pub trait outside,
//But seems like the cmd()... still can be called outside. //FIXME
pub trait FirstCmd: private::Sealed {
    #[doc(hidden)]
    fn cmd(&self) -> &str;
    #[doc(hidden)]
    fn session(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct GetCmd {
    cmd: String,
    session: String,
}

impl FirstCmd for GetCmd {
    #[doc(hidden)]
    fn cmd(&self) -> &str {
        &self.cmd
    }
    #[doc(hidden)]
    fn session(&self) -> &str {
        &self.session
    }
}

//this macro repeat codes above. to create a first cmd then
//implement FirstCmd trait for it.
macro_rules! its_first_cmd {
    ($first_cmd:ident) => {
        #[derive(Debug, Clone)]
        pub struct $first_cmd {
            cmd: String,
            session: String,
        }

        impl FirstCmd for $first_cmd {
            #[doc(hidden)]
            fn cmd(&self) -> &str {
                &self.cmd
            }
            #[doc(hidden)]
            fn session(&self) -> &str {
                &self.session
            }
        }
    };
}

its_first_cmd!(CreateCmd);
its_first_cmd!(ListCmd);

//Maybe I can generic on some of second cmd's method, they seems like do same thing.
//TODO
impl GetCmd {
    pub fn account(&self) -> AccountCmd {
        let flags: Vec<String> = Vec::new();
        AccountCmd {
            first: self.clone(),
            cmd: "account".to_string(),
            flags,
        }
    }

    ///this method return items' fields of website,username,password
    pub fn item_lite(&self, item_name: &str) -> ItemLiteCmd {
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

    pub fn item(&self, item_name: &str) -> GetItemCmd {
        let flags: Vec<String> = vec![item_name.to_string()];
        GetItemCmd {
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

impl ListCmd {
    pub fn documents(&self) -> ListDocumentsCmd {
        let flags: Vec<String> = Vec::new();
        ListDocumentsCmd {
            first: self.clone(),
            cmd: "documents".to_string(),
            flags,
        }
    }

    pub fn items(&self) -> ListItemsCmd {
        let flags: Vec<String> = Vec::new();
        ListItemsCmd {
            first: self.clone(),
            cmd: "items".to_string(),
            flags,
        }
    }
}

//The Sealed Pattern can prevent others impl this pub trait outside,
//But seems like the first()... still can be called outside. //FIXME
#[async_trait::async_trait]
pub trait SecondCmd: private::Sealed {
    type Output: DeserializeOwned;
    type First: FirstCmd + Clone;

    #[doc(hidden)]
    fn first(&self) -> &Self::First;
    #[doc(hidden)]
    fn cmd(&self) -> &str;
    #[doc(hidden)]
    fn flags(&self) -> Vec<String>;

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
            self.first().cmd().to_string(),
            self.cmd().to_string(),
            "--session".to_string(),
            self.first().session().trim().to_string(),
        ];
        if !self.flags().is_empty() {
            self.flags()
                .into_iter()
                .for_each(|flag| args.push(flag.to_string()))
        }
        let out_str: &str = &exec_command(args).await?;
        Ok(serde_json::from_str(out_str)?)
    }
}

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

    #[doc(hidden)]
    fn first(&self) -> &GetCmd {
        &self.first
    }

    #[doc(hidden)]
    fn cmd(&self) -> &str {
        &self.cmd
    }

    #[doc(hidden)]
    fn flags(&self) -> Vec<String> {
        self.flags.clone()
    }
}

//This macro repeat above codes. To create a new second cmd struct
//and implement SecondCmd trait for it.
macro_rules! its_second_cmd {
    ($first_cmd:ident,$second_cmd:ident,$output:ident) => {
        #[derive(Debug)]
        pub struct $second_cmd {
            first: $first_cmd,
            cmd: String,
            flags: Vec<String>,
        }

        #[async_trait::async_trait]
        impl SecondCmd for $second_cmd {
            type Output = output::$output;
            type First = $first_cmd;
            #[doc(hidden)]
            fn first(&self) -> &$first_cmd {
                &self.first
            }
            #[doc(hidden)]
            fn cmd(&self) -> &str {
                &self.cmd
            }
            #[doc(hidden)]
            fn flags(&self) -> Vec<String> {
                self.flags.clone()
            }
        }
    };
}

its_second_cmd!(GetCmd, ItemLiteCmd, ItemLite);
its_second_cmd!(GetCmd, GetDocumentCmd, Value);
its_second_cmd!(GetCmd, GetTotpCmd, Value);
its_second_cmd!(GetCmd, GetItemCmd, GetItem);
its_second_cmd!(CreateCmd, CreateDocumentCmd, CreateDocument);
its_second_cmd!(ListCmd, ListDocumentsCmd, ListDocuments);
its_second_cmd!(ListCmd, ListItemsCmd, ListItems);

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

mod private {
    pub trait Sealed {}

    impl Sealed for crate::AccountCmd {}
    impl Sealed for crate::ListItemsCmd {}
    impl Sealed for crate::ItemLiteCmd {}
    impl Sealed for crate::GetDocumentCmd {}
    impl Sealed for crate::GetTotpCmd {}
    impl Sealed for crate::GetItemCmd {}
    impl Sealed for crate::CreateDocumentCmd {}
    impl Sealed for crate::ListDocumentsCmd {}
    impl Sealed for crate::GetCmd {}
    impl Sealed for crate::CreateCmd {}
    impl Sealed for crate::ListCmd {}
}
