use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use relative_path::RelativePath;

pub struct ServerEntry<'a> {
    pub entry: &'a str,
    pub output: Option<&'a str>,
    pub manifest: Option<&'a str>,
}

impl<'a> ServerEntry<'a> {
    pub fn new(entry: &'a str) -> ServerEntry<'a> {
        ServerEntry {
            entry,
            output: Some("server"),
            manifest: Some(".vite/manifest.json"),
        }
    }

    pub fn output(mut self, path: &'a str) -> Self {
        self.output = Some(path);
        self
    }

    pub fn manifest(mut self, path: &'a str) -> Self {
        self.manifest = Some(path);
        self
    }

    pub fn entry_path(&self, root: &Path) -> PathBuf {
        root.join(self.entry)
    }

    pub fn output_path(&self, root: &Path) -> PathBuf {
        root.join(self.output.unwrap_or("client"))
    }

    pub fn manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.manifest.unwrap_or(".vite/manifest.json"))
    }
}

pub struct ClientEntry<'a> {
    pub entry: &'a str,
    pub output: Option<&'a str>,
    pub manifest: Option<&'a str>,
    pub ssr_manifest: Option<&'a str>,
}

impl<'a> ClientEntry<'a> {
    pub fn new(entry: &'a str) -> ClientEntry<'a> {
        ClientEntry {
            entry,
            output: None,
            manifest: None,
            ssr_manifest: None,
        }
    }

    pub fn output(mut self, path: &'a str) -> Self {
        self.output = Some(path);
        self
    }

    pub fn manifest(mut self, path: &'a str) -> Self {
        self.manifest = Some(path);
        self
    }

    pub fn ssr_manifest(mut self, path: &'a str) -> Self {
        self.ssr_manifest = Some(path);
        self
    }

    pub fn entry_path(&self, root: &Path) -> PathBuf {
        root.join(self.entry)
    }

    pub fn output_path(&self, root: &Path) -> PathBuf {
        root.join(self.output.unwrap_or("client"))
    }

    pub fn manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.manifest.unwrap_or(".vite/manifest.json"))
    }

    pub fn ssr_manifest_path(&self, root: &Path) -> PathBuf {
        root.join(self.ssr_manifest.unwrap_or(".vite/ssr-manifest.json"))
    }
}

pub struct ViteOptions<'a> {
    pub(crate) path: PathBuf,
    pub(crate) server_manifest: Option<&'a str>,
    pub(crate) client_manifest: Option<&'a str>,
    pub(crate) ssr_manifest: Option<&'a str>,
}

impl<'a> ViteOptions<'a> {
    pub fn new<P: Into<PathBuf>>(path: P) -> ViteOptions<'a> {
        ViteOptions {
            path: path.into(),
            server_manifest: None,
            client_manifest: None,
            ssr_manifest: None,
        }
    }

    pub fn client_manifest(mut self, path: &'a str) -> Self {
        self.client_manifest = Some(path);
        self
    }

    pub(crate) fn get_server_manifest(&self) -> PathBuf {
        RelativePath::new(self.server_manifest.unwrap_or("server/.vite/manifest.json"))
            .to_logical_path(&self.path)
    }

    pub(crate) fn get_client_manifest(&self) -> PathBuf {
        RelativePath::new(self.client_manifest.unwrap_or("client/.vite/manifest.json"))
            .to_logical_path(&self.path)
    }

    pub(crate) fn get_ssr_manifest(&self) -> PathBuf {
        RelativePath::new(
            self.ssr_manifest
                .unwrap_or("client/.vite/ssr-manifest.json"),
        )
        .to_logical_path(&self.path)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    pub file: String,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default, rename = "dynamicImports")]
    pub dynamic_imports: Vec<String>,
    #[serde(default, rename = "isEntry")]
    pub is_entry: bool,
    #[serde(default)]
    pub imports: Vec<String>,
    // src: String,
}

pub type Manifest = HashMap<String, ManifestEntry>;

pub type SSRManifest = HashMap<String, Vec<String>>;
