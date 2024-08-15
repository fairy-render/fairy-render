use std::path::PathBuf;

use fairy_render::Renderer;
use reggie::{Body, Request};

use crate::{
    util::load_json, Asset, AssetKind, Entry, FairyResult, Manifest, SSRManifest, ViteError,
    ViteOptions,
};

#[derive(Clone, Debug)]
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

impl From<Entry> for ViteEntry {
    fn from(value: Entry) -> Self {
        ViteEntry {
            client: value.client.into(),
            server: value.server,
        }
    }
}

pub struct ViteResolver {
    ssrmanifest: SSRManifest,
    server_manifest: Manifest,
    client_manifest: Manifest,
    root: PathBuf,
}

impl ViteResolver {
    pub async fn render<B: Into<Body>, R>(
        &self,
        entry: impl Into<ViteEntry>,
        req: Request<B>,
        renderer: &R,
    ) -> Result<FairyResult, ViteError>
    where
        R: Renderer,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let vite_entry: ViteEntry = entry.into();

        let Some(entry) = self.server_manifest.get(&vite_entry.server) else {
            panic!("entry not found: {:?}", vite_entry);
        };

        if !entry.is_entry {
            panic!("entry is not an entrypoint");
        }

        let mut assets = Vec::default();

        if let Some(client) = vite_entry.client {
            let Some(client_entry) = self.client_manifest.get(&client) else {
                panic!("client entry does not exists")
            };

            assets.push(Asset {
                file: client_entry.file.clone(),
                kind: AssetKind::Script,
            });

            for css in &client_entry.css {
                assets.push(Asset {
                    file: css.clone(),
                    kind: AssetKind::Styling,
                });
            }
        }

        let path = format!("./server/{}", entry.file);

        let result = renderer
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

impl<'a> ViteOptions<'a> {
    pub async fn build(self) -> Result<ViteResolver, ViteError> {
        let client_manifest: Manifest = load_json(&self.get_client_manifest()).await?;
        let server_manifest: Manifest = load_json(&self.get_server_manifest()).await?;
        let ssrmanifest = load_json(&self.get_ssr_manifest()).await?;

        Ok(ViteResolver {
            ssrmanifest,
            server_manifest,
            client_manifest,
            root: self.path,
        })
    }
}
