use std::{collections::hash_map::Keys, sync::Arc};

use crate::{
    config::ViteConfig, vite::Vite, vite_resolver::ViteEntry, Entry, EntryValue, FairyResult,
    ViteError,
};
use fairy_render::{
    quick::{Quick, QuickFactory},
    RendererFactory,
};
use reggie::{factory_arc, Body, HttpClient, HttpClientFactory, Request};

pub struct Fairy {
    pub config: ViteConfig,
    pub vite: Arc<Vite>,
    pub vm: Option<Arc<Quick>>,
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

        let vm = factory.create(factory_arc(http)).await.unwrap();

        let vite = Vite::new(&config, false).await?;

        Ok(Fairy {
            config,
            vm: Some(Arc::new(vm)),
            vite: Arc::new(vite),
        })
    }

    pub fn dev(config: ViteConfig) -> Result<Fairy, ViteError> {
        Ok(Fairy {
            vite: Vite::dev(&config).into(),
            config,
            vm: None,
        })
    }

    pub fn entries(&self) -> Option<Keys<'_, String, Entry>> {
        match &self.config.entries {
            EntryValue::Entry(_) => None,
            EntryValue::Many(m) => Some(m.keys()),
        }
    }

    pub fn config(&self) -> &ViteConfig {
        &self.config
    }

    pub fn create_renderer<'a>(&self, entry: impl Into<Option<&'a str>>) -> FairyRenderer {
        let entry = entry.into();

        let Some(_) = self.config.get_entry(entry) else {
            panic!("entry not found: {entry:?}");
        };

        FairyRenderer {
            vite: self.vite.clone(),
            vm: self.vm.clone(),
            entry: entry.map(|m| m.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct FairyRenderer {
    pub vite: Arc<Vite>,
    pub vm: Option<Arc<Quick>>,
    pub entry: Option<String>,
}

impl FairyRenderer {
    pub async fn render<B: Into<Body>>(&self, req: Request<B>) -> Result<FairyResult, ViteError> {
        self.vite
            .render(self.entry.as_ref().map(|m| m.as_str()), req, &self.vm)
            .await
    }
}
