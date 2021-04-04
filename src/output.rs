use chrono::prelude::*;
use serde::{Deserialize, Serialize};
pub use serde_json::Value;

pub type ListDocuments = Vec<ListDocument>;

pub type ListItems = Vec<ListItem>;

pub type ListUsers = Vec<ListUser>;
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemLite {
    pub website: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetItem {
    pub uuid: String,
    #[serde(alias = "templateUuid")]
    pub template_uuid: String,
    pub trashed: String,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "updatedAt")]
    #[serde(with = "date_format")]
    pub update_at: DateTime<Local>,
    #[serde(alias = "changerUuid")]
    pub changer_uuid: String,
    #[serde(alias = "itemVersion")]
    pub item_version: usize,
    #[serde(alias = "vaultUuid")]
    pub vault_uuid: String,
    pub details: Value, // this field is a serde_json::Value because its content would change depend on the queried item.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub state: String,
    pub avatar: String,
    pub domain: String,
    #[serde(alias = "attrVersion")]
    pub attr_version: usize,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "baseAvatarURL")]
    pub base_avatar_url: String,
    #[serde(alias = "baseAttachmentURL")]
    pub base_attachment_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDocument {
    pub uuid: String,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "updatedAt")]
    #[serde(with = "date_format")]
    pub update_at: DateTime<Local>,
    #[serde(alias = "vaultUuid")]
    pub vault_uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListDocument {
    pub uuid: String,
    #[serde(alias = "templateUuid")]
    pub template_uuid: String,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "updatedAt")]
    #[serde(with = "date_format")]
    pub update_at: DateTime<Local>,
    #[serde(alias = "itemVersion")]
    pub item_version: usize,
    #[serde(alias = "vaultUuid")]
    pub vault_uuid: String,
    pub overview: Value, // this field is a serde_json::Value because its content would change depend on the queried item.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListUser {
    pub uuid: String,
    #[serde(alias = "firstName")]
    pub first_name: String,
    #[serde(alias = "lastName")]
    pub last_name: String,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub state: String,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUser {
    pub uuid: String,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "updatedAt")]
    #[serde(with = "date_format")]
    pub update_at: DateTime<Local>,
    #[serde(alias = "lastAuthAt")]
    #[serde(with = "date_format")]
    pub last_auth_at: DateTime<Local>,
    #[serde(alias = "firstName")]
    pub first_name: String,
    #[serde(alias = "lastName")]
    pub last_name: String,
    pub name: String,
    pub email: String,
    #[serde(alias = "attrVersion")]
    pub attr_version: usize,
    #[serde(alias = "keysetVersion")]
    pub keyset_version: usize,
    pub language: String,
    #[serde(alias = "accountKeyFormat")]
    pub account_key_format: String,
    #[serde(alias = "accountKeyUuid")]
    pub account_key_uuid: String,
    #[serde(alias = "combinedPermissions")]
    pub combined_permissions: usize,
    pub avatar: String,
    pub state: String,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListItem {
    pub uuid: String,
    #[serde(alias = "templateUuid")]
    pub template_uuid: String,
    pub trashed: String,
    #[serde(alias = "createdAt")]
    #[serde(with = "date_format")]
    pub create_at: DateTime<Local>,
    #[serde(alias = "updatedAt")]
    #[serde(with = "date_format")]
    pub update_at: DateTime<Local>,
    #[serde(alias = "changerUuid")]
    pub changer_uuid: String,
    #[serde(alias = "itemVersion")]
    pub item_version: usize,
    #[serde(alias = "vaultUuid")]
    pub vault_uuid: String,
    pub overview: Value, // this field is a serde_json::Value because its content would change depend on the queried item.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteItem {
    pub field: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteDocument {
    pub field: String,
}

//this mod helped to deserialize json string to chrono::DateTime.
//And it was copied from StackOverflow!
mod date_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.f%Z";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Local
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
