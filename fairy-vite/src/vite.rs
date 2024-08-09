use fairy_render::Renderer;
use reggie::{http::Request, Body};

use crate::{
    error::ViteError,
    result::{Asset, AssetKind, FairyResult},
    vite_options::ViteOptions,
    vite_resolver::{ViteEntry, ViteResolver},
    ViteConfig,
};

enum Mode {
    Prod(ViteResolver),
    Dev(ViteConfig),
}

pub struct Vite {
    mode: Mode,
}

impl Vite {
    pub async fn new(config: &ViteConfig, dev: bool) -> Result<Vite, ViteError> {
        let vite = if dev {
            Self::dev(config)
        } else {
            let opts = ViteOptions::new(config.root()).client_manifest(&config.client_manifest);
            let resolver = opts.build().await?;

            Vite {
                mode: Mode::Prod(resolver),
            }
        };

        Ok(vite)
    }

    pub fn dev(config: &ViteConfig) -> Vite {
        Vite {
            mode: Mode::Dev(config.clone()),
        }
    }

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
        match &self.mode {
            Mode::Dev(config) => {
                let entry: ViteEntry = entry.into();
                let Some(entry) = config.get_entry(entry.client.as_ref().map(|m| m.as_str()))
                else {
                    panic!("entry not found: {entry:?}");
                };

                Ok(FairyResult {
                    head: Vec::new(),
                    assets: vec![Asset {
                        kind: AssetKind::Script,
                        file: format!("http://localhost:{}/{}", config.port, entry.client),
                    }],
                    content: Vec::new(),
                })
            }
            Mode::Prod(resolver) => resolver.render(entry, req, renderer).await,
        }
    }
}
