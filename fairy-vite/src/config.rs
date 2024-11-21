use crate::{util::load_json, ViteError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ViteConfig {
    pub assets: String,
    pub assets_path: String,
    pub base: String,
    pub client_manifest: String,
    pub entries: EntryValue,
    pub port: u16,
    pub root: String,
    pub server_manifest: String,
    pub ssr_manifest: String,
    pub work_dir: String,
}

impl ViteConfig {
    pub fn get_entry(&self, name: Option<&str>) -> Option<&Entry> {
        match (name, &self.entries) {
            (Some(name), EntryValue::Many(map)) => map.get(name),
            (None, EntryValue::Entry(entry)) => Some(entry),
            _ => None,
        }
    }

    pub fn work_dir(&self) -> &Path {
        Path::new(&self.work_dir)
    }

    /// Path to client side assets
    pub fn assets(&self) -> PathBuf {
        self.root().join("client").join(&self.assets)
    }

    /// The build root (eg. dist)
    pub fn root(&self) -> PathBuf {
        self.work_dir().join(&self.root)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum EntryValue {
    Entry(Entry),
    Many(HashMap<String, Entry>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    pub client: String,
    pub server: String,
}

impl ViteConfig {
    pub async fn load(path: &Path) -> Result<ViteConfig, ViteError> {
        load_json(path).await
    }
}
