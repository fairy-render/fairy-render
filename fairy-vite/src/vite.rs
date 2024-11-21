use fairy_render::Renderer;
use reggie::{http::Request, Body};

use crate::{
    error::ViteError,
    result::{Asset, AssetKind, FairyResult},
    vite_options::ViteOptions,
    vite_resolver::ViteResolver,
    ViteConfig,
};

enum Mode {
    Prod(ViteResolver),
    Dev,
}

pub struct Vite {
    mode: Mode,
    config: ViteConfig,
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
                config: config.clone(),
            }
        };

        Ok(vite)
    }

    pub fn dev(config: &ViteConfig) -> Vite {
        Vite {
            mode: Mode::Dev,
            config: config.clone(),
        }
    }

    pub async fn render<B: Into<Body>, R>(
        &self,
        entry: Option<&str>,
        req: Request<B>,
        renderer: &R,
    ) -> Result<FairyResult, ViteError>
    where
        R: Renderer,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let Some(entry) = self.config.get_entry(entry) else {
            panic!("entry not found: {entry:?}");
        };

        match &self.mode {
            Mode::Dev => Ok(FairyResult {
                head: Vec::new(),
                assets: vec![Asset {
                    kind: AssetKind::Script,
                    file: format!("http://localhost:{}/{}", self.config.port, entry.client),
                }],
                content: Vec::new(),
            }),
            Mode::Prod(resolver) => resolver.render(entry.clone(), req, renderer).await,
        }
    }
}
