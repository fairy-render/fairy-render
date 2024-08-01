use std::path::PathBuf;

use fairy_render::{Renderer, RendererFactory};
use reggie::{factory_arc, http::Request, Body, HttpClient, HttpClientFactory};

use crate::{
    error::ViteError,
    result::{Asset, AssetKind, FairyResult},
    util::load_json,
    vite_options::{Manifest, SSRManifest, ViteOptions},
};

pub struct Vite<R> {
    ssrmanifest: SSRManifest,
    server_manifest: Manifest,
    client_manifest: Manifest,
    renderer: R,
    root: PathBuf,
}

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

impl<R: Renderer> Vite<R>
where
    R::Error: std::error::Error + Send + Sync + 'static,
{
    pub async fn render<B: Into<Body>>(
        &self,
        entry: impl Into<ViteEntry>,
        req: Request<B>,
    ) -> Result<FairyResult, ViteError> {
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

impl<'a> ViteOptions<'a> {
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
        let client_manifest: Manifest = load_json(&self.get_client_manifest()).await?;
        let server_manifest: Manifest = load_json(&self.get_server_manifest()).await?;
        let ssrmanifest = load_json(&self.get_ssr_manifest()).await?;

        let renderer = renderer.create(factory_arc(http_factory)).await.unwrap();

        Ok(Vite {
            ssrmanifest,
            server_manifest,
            client_manifest,
            renderer,
            root: self.path,
            // asset_path,
            // asset_base,
        })
    }
}
