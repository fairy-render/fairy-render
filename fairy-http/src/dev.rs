use axum::{http::Request, response::Response};
use fairy_vite::{Asset, AssetKind, Entry, FairyResult, ViteConfig};
use reggie::Body;
use std::{
    convert::Infallible,
    future::{ready, Ready},
    sync::Arc,
    task::Poll,
};
use tower_service::Service;

use crate::{template::Template, ViteService};

#[derive(Clone)]
pub struct ViteDevService {
    config: Arc<ViteConfig>,
    template: Arc<dyn Template + Send + Sync>,
    entry: Entry,
}

impl ViteDevService {
    pub fn new<'a, T>(
        config: ViteConfig,
        template: T,
        entry: impl Into<Option<&'a str>>,
    ) -> ViteDevService
    where
        T: Template + Send + Sync + 'static,
    {
        let Some(entry) = config.get_entry(entry.into()) else {
            panic!("no entry")
        };

        ViteDevService {
            entry: entry.clone(),
            config: Arc::new(config),
            template: Arc::new(template),
        }
    }
}

impl<B> Service<Request<B>> for ViteDevService {
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let result = FairyResult {
            head: Vec::new(),
            assets: vec![Asset {
                kind: AssetKind::Script,
                file: format!(
                    "http://localhost:{}/{}",
                    self.config.port, self.entry.client
                ),
            }],
            content: Vec::new(),
        };

        let output = self.template.render(req.uri().clone(), Ok(result));

        let resp = Response::builder()
            .header("Content-Type", "text/html")
            .status(200)
            .body(Body::from(output))
            .expect("build response");

        ready(Ok(resp))
    }
}
