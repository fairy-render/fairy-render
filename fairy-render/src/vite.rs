use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use futures_core::Future;
use reggie::{factory_arc, factory_box, http::Request, Body, HttpClient, HttpClientFactory};
use relative_path::RelativePathBuf;

use crate::{
    load_json,
    renderer::{Renderer, RendererFactory},
    result::{Asset, AssetKind, FairyResult},
};

#[derive(Debug, thiserror::Error)]
pub enum ViteError {
    #[error("render error: {0}")]
    Render(Box<dyn std::error::Error + Send + Sync>),
    #[error("manifest not found: {path} or errored: {error:?}")]
    Manifest {
        path: String,
        error: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

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
    pub path: PathBuf,
    pub server: ServerEntry<'a>,
    pub client: ClientEntry<'a>,
}

impl<'a> ViteOptions<'a> {
    #[cfg(feature = "reqwest")]
    pub async fn build<T>(self, renderer: T) -> Result<Vite<T::Renderer>, ViteError>
    where
        T: RendererFactory,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        self.build_with(renderer, reggie::Reqwest::default()).await
    }

    pub async fn build_with<T, F>(
        self,
        renderer: T,
        http_factory: F,
    ) -> Result<Vite<T::Renderer>, ViteError>
    where
        T: RendererFactory,
        T::Error: std::error::Error + Send + Sync + 'static,
        F: HttpClientFactory + Send + Sync + 'static,
        F::Client<Body>: Send + Sync + 'static,
        for<'b> <F::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
        <F::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    {
        let path = self.path;

        let client_path = self.client.output_path(&path);
        let server_path = self.server.output_path(&path);

        let client_manifest_path = self.client.manifest_path(&client_path);
        let client_ssr_manifest_path = self.client.ssr_manifest_path(&client_path);

        let server_manifest_path = self.server.manifest_path(&server_path);

        let client_manifest: Manifest = load_json(&client_manifest_path).await?;

        let server_manifest: Manifest = load_json(&server_manifest_path).await?;

        let ssrmanifest = load_json(&client_ssr_manifest_path).await?;

        let renderer = renderer.create(factory_arc(http_factory)).await.unwrap();
        // let asset_path = client_path.join("assets");
        // let asset_base = "/assets".to_string();

        Ok(Vite {
            ssrmanifest,
            server_manifest,
            client_manifest,
            renderer,
            root: path,
            // asset_path,
            // asset_base,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestEntry {
    file: String,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default, rename = "dynamicImports")]
    dynamic_imports: Vec<String>,
    #[serde(default, rename = "isEntry")]
    is_entry: bool,
    #[serde(default)]
    imports: Vec<String>,
    src: String,
}

type Manifest = HashMap<String, ManifestEntry>;

type SSRManifest = HashMap<String, Vec<String>>;

pub struct Vite<R> {
    ssrmanifest: SSRManifest,
    server_manifest: Manifest,
    client_manifest: Manifest,
    renderer: R,
    root: PathBuf,
    // asset_path: PathBuf,
    // asset_base: String,
}

#[derive(Clone)]
pub struct ViteEntry {
    pub client: Option<String>,
    pub server: String,
}

impl From<String> for ViteEntry {
    fn from(value: String) -> Self {
        ViteEntry {
            client: None,
            server: value,
        }
    }
}

impl<'a> From<&'a str> for ViteEntry {
    fn from(value: &'a str) -> Self {
        value.to_string().into()
    }
}

impl<R: Renderer> Vite<R>
where
    R::Error: std::error::Error + Send + Sync + 'static,
{
    // pub fn assets_path(&self) -> &Path {
    //     &self.asset_path
    // }

    // pub fn asset_base(&self) -> &str {
    //     &self.asset_base
    // }

    pub async fn render<B: Into<Body>>(
        &self,
        entry: impl Into<ViteEntry>,
        req: Request<B>,
    ) -> Result<FairyResult, ViteError> {
        let vite_entry: ViteEntry = entry.into();

        let Some(entry) = self.server_manifest.get(&vite_entry.server) else {
            panic!("entry not found");
        };

        if !entry.is_entry {
            panic!("entry is not an entrypoint");
        }

        let mut assets = Vec::default();

        if let Some(client) = vite_entry.client {
            let Some(client_entry) = self.client_manifest.get(&client) else {
                panic!("client entry does not exists")
            };

            let mut assets = vec![Asset {
                file: client_entry.file.clone(),
                kind: AssetKind::Script,
            }];

            for css in &client_entry.css {
                assets.push(Asset {
                    file: css.clone(),
                    kind: AssetKind::Styling,
                });
            }
        }

        let path = format!("./server/{}", entry.file);

        let result = self
            .renderer
            .render(path.into(), req.map(Into::into))
            .await
            .map_err(|err| ViteError::Render(Box::new(err)))?;

        for file in result.assets {
            let Some(files) = self.ssrmanifest.get(&file) else {
                println!("could not find {file} in manifest");
                continue;
            };

            for file in files {
                let file_path = relative_path::RelativePath::new(&file);
                let kind = match file_path.extension() {
                    Some("js" | "mjs" | "ts" | "tsx") => AssetKind::Script,
                    Some("css") => AssetKind::Styling,
                    _ => AssetKind::Unknown,
                };

                let file = if file.starts_with("/") {
                    file.chars().skip(1).collect::<String>()
                } else {
                    file.clone()
                };

                assets.push(Asset { file, kind })
            }
        }

        Ok(FairyResult {
            content: result.content.to_vec(),
            head: result.head,
            assets,
        })
    }
}
