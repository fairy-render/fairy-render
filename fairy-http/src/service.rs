use std::{
    convert::Infallible,
    path::{Path, PathBuf},
};

use crate::RenderService;

use super::template::Template;
use axum::{body::Bytes, routing::RouterIntoService, BoxError, Router};
use fairy_render::{
    quick::Quick,
    vite::{Vite, ViteEntry},
};
use reggie::http::{Request, Response};
use tower::{Layer, Service, ServiceExt};
use tower_http::services::ServeDir;

pub struct ViteService<B> {
    pub(crate) inner: RouterIntoService<B, ()>,
}

impl<B> Clone for ViteService<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B> ViteService<B> {
    pub fn new<T>(
        vite: Vite<Quick>,
        entry: ViteEntry,
        template: T,
        assets_path: impl AsRef<Path>,
        asset_base: &str,
    ) -> ViteService<B>
    where
        T: Template + Send + Sync + 'static,
    {
        let mut router = Router::new();

        router = router.nest_service(&asset_base, ServeDir::new(assets_path));

        let render = RenderService::new(vite, entry, template);

        ViteService {
            inner: router.fallback_service(render).into_service(),
        }
    }
}

impl<B> Service<Request<B>> for ViteService<B>
where
    B: reggie::http_body::Body<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    type Response = axum::response::Response;

    type Error = Infallible;

    type Future = <RouterIntoService<B, ()> as Service<Request<B>>>::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        self.inner.call(req)
    }
}
