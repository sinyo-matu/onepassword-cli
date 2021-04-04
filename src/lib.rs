pub mod error;
pub mod output;
mod tests;
use chrono::{prelude::*, Duration};
use error::Error;

use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::sealed::{FirstCmd, SecondCmd};

pub type Result<T> = std::result::Result<T, Error>;

//OpCLI have expiration_time field what is the token's expiration time.
//Intent to implement some method to auto renew the token. //TODO
#[derive(Clone)]
pub struct OpCLI {
    expiration_time: DateTime<Utc>,
    session: String,
}

impl OpCLI {
    #[inline]
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

    #[inline]
    pub fn list(&self) -> ListCmd {
        ListCmd {
            cmd: "list".to_string(),
            session: self.session.to_string(),
        }
    }

    #[inline]
    pub fn delete(&self) -> DeleteCmd {
        DeleteCmd {
            cmd: "delete".to_string(),
            session: self.session.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetCmd {
    cmd: String,
    session: String,
}

impl sealed::FirstCmd for GetCmd {
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

        impl sealed::FirstCmd for $first_cmd {
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
its_first_cmd!(DeleteCmd);

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
    pub fn item_lite(&self, item: &str) -> ItemLiteCmd {
        let flags: Vec<String> = vec![
            item.to_string(),
            "--fields".to_string(),
            "website,username,password".to_string(),
        ];
        ItemLiteCmd {
            first: self.clone(),
            cmd: "item".to_string(),
            flags,
        }
    }

    pub fn item(&self, item: &str) -> GetItemCmd {
        let flags: Vec<String> = vec![item.to_string()];
        GetItemCmd {
            first: self.clone(),
            cmd: "item".to_string(),
            flags,
        }
    }

    pub fn document(&self, doc: &str) -> GetDocumentCmd {
        let flags: Vec<String> = vec![doc.to_string()];
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

    pub fn user(&self, uuid: &str) -> GetUserCmd {
        let flags: Vec<String> = vec![uuid.to_string()];

        GetUserCmd {
            first: self.clone(),
            cmd: "user".to_string(),
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

    pub fn users(&self) -> ListUsersCmd {
        let flags: Vec<String> = Vec::new();
        ListUsersCmd {
            first: self.clone(),
            cmd: "users".to_string(),
            flags,
        }
    }
}

impl DeleteCmd {
    pub fn item(&self) -> DeleteItemCmd {
        let flags: Vec<String> = Vec::new();
        DeleteItemCmd {
            first: self.clone(),
            cmd: "item".to_string(),
            flags,
        }
    }

    pub fn document(&self, doc: &str) -> DeleteDocumentCmd {
        let flags: Vec<String> = vec![doc.to_string()];
        DeleteDocumentCmd {
            first: self.clone(),
            cmd: "document".to_string(),
            flags,
        }
    }
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
        if out_str.len() == 0 {
            return Ok(serde_json::from_str("{\"field\":\"ok\"}")?);
        }
        Ok(serde_json::from_str(out_str)?)
    }
}

impl<T: SecondCmd> SecondCmdExt for T {}

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
its_second_cmd!(GetCmd, GetUserCmd, GetUser);
its_second_cmd!(CreateCmd, CreateDocumentCmd, CreateDocument);
its_second_cmd!(ListCmd, ListDocumentsCmd, ListDocuments);
its_second_cmd!(ListCmd, ListItemsCmd, ListItems);
its_second_cmd!(ListCmd, ListUsersCmd, ListUsers);
its_second_cmd!(DeleteCmd, DeleteItemCmd, DeleteItem);
its_second_cmd!(DeleteCmd, DeleteDocumentCmd, DeleteDocument);

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
    match std_err.trim() {
        err if err.contains("401") => Err(Error::OPSignInError("Wrong password".to_string())),
        err if err.contains("Account not found") => Err(Error::OPSignInError(
            "Account does not exist,may be you should firstly setup 1password-cli.".to_string(),
        )),
        _ => Ok(()),
    }
}

#[inline]
async fn handle_op_exec_error(std_err: String) -> std::result::Result<(), Error> {
    match std_err.trim() {
        err if err.contains("doesn't seem to be an item") => {
            Err(Error::ItemQueryError("Item not founded".to_string()))
        }
        err if err.contains("Invalid session token") => {
            Err(Error::ItemQueryError("In valid session token".to_string()))
        }
        err if err.contains("More than one item matches") => Err(Error::ItemQueryError(
            "More than one item matches,Please specify one by uuid".to_string(),
        )),
        _ => Ok(()),
    }
}

//why I need this: Cause of SecondCmdExt need explicit scope in.
//So this will impl a casting method for the struct who
//implemented SecondCmdExt. now user do not need `use crate::SecondCmdExt`
macro_rules! impl_casting_method {
    ($($ObjName:ident),* $(,)?) => {
        $(
            impl $ObjName {

                pub async fn run(&self) -> Result<<Self as SecondCmd>::Output> {
                    <Self as SecondCmdExt>::run(self).await
                }

                pub fn add_flag(&mut self, flags: &[&str]) -> &Self {
                    <Self as SecondCmdExt>::add_flag(self, flags)
                }
            }
        )*
    };
}

impl_casting_method!(
    ItemLiteCmd,
    GetDocumentCmd,
    GetTotpCmd,
    GetItemCmd,
    CreateDocumentCmd,
    ListDocumentsCmd,
    ListItemsCmd,
    AccountCmd
);

mod sealed {
    use serde::de::DeserializeOwned;

    pub trait FirstCmd {
        #[doc(hidden)]
        fn cmd(&self) -> &str;
        #[doc(hidden)]
        fn session(&self) -> &str;
    }

    #[async_trait::async_trait]
    pub trait SecondCmd {
        type Output: DeserializeOwned;
        type First: FirstCmd + Clone;

        #[doc(hidden)]
        fn first(&self) -> &Self::First;
        #[doc(hidden)]
        fn cmd(&self) -> &str;
        #[doc(hidden)]
        fn flags(&self) -> Vec<String>;
    }
}
