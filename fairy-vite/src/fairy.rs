use std::sync::Arc;

use crate::{
    config::ViteConfig, vite::Vite, Asset, AssetKind, FairyResult, ViteEntry, ViteError,
    ViteOptions,
};
use fairy_render::{
    quick::{Quick, QuickFactory},
    Renderer,
};
use reggie::{Body, HttpClient, HttpClientFactory, Request};

enum Mode {
    Prod {
        vite: Arc<Vite<Quick>>,
        entry: ViteEntry,
    },
    Dev(FairyResult),
}

pub struct Fairy {
    pub(crate) config: ViteConfig,
    pub(crate) vite: Option<Arc<Vite<Quick>>>,
}

impl Fairy {
    pub async fn new<T: HttpClientFactory>(config: ViteConfig, http: T) -> Result<Fairy, ViteError>
    where
        T: HttpClientFactory + Send + Sync + 'static,
        T::Client<Body>: Send + Sync + 'static,
        for<'b> <T::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
        <T::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    {
        let factory = QuickFactory::default().search_path(config.root());

        let opts = ViteOptions::new(config.root()).client_manifest(&config.client_manifest);
        let vite = opts.build_with(factory, http).await?;

        Ok(Fairy {
            config: config,
            vite: Some(Arc::new(vite)),
        })
    }

    pub fn create_renderer<'a>(&self, entry: impl Into<Option<&'a str>>) -> FairyRenderer {
        let entry = entry.into();

        let Some(entry) = self.config.get_entry(entry) else {
            panic!("entry not found: {entry:?}");
        };

        let mode = if let Some(vite) = &self.vite {
            Mode::Prod {
                vite: vite.clone(),
                entry: ViteEntry {
                    client: entry.client.clone().into(),
                    server: entry.server.clone(),
                },
            }
        } else {
            Mode::Dev(FairyResult {
                head: Vec::new(),
                assets: vec![Asset {
                    kind: AssetKind::Script,
                    file: format!("http://localhost:{}/{}", self.config.port, entry.client),
                }],
                content: Vec::new(),
            })
        };

        FairyRenderer { mode }
    }
}

pub struct FairyRenderer {
    mode: Mode,
}

impl FairyRenderer {
    pub async fn render<B: Into<Body>>(&self, req: Request<B>) -> Result<FairyResult, ViteError> {
        match self.mode {
            Mode::Dev(ref ret) => Ok(ret.clone()),
            Mode::Prod {
                ref vite,
                ref entry,
            } => vite.render(entry.clone(), req).await,
        }
    }
}
