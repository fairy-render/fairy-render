use std::{convert::Infallible, sync::Arc};

use futures_core::{future::BoxFuture, Future};
use reggie::{bytes::Bytes, http::Request, Body, SharedClientFactory};
use relative_path::RelativePathBuf;

#[derive(Debug)]
pub struct RenderResult {
    pub content: Bytes,
    pub assets: Vec<String>,
    pub head: Vec<String>,
}

pub trait Renderer {
    type Error;
    fn render<'a>(
        &'a self,
        path: RelativePathBuf,
        req: Request<Body>,
    ) -> BoxFuture<'a, Result<RenderResult, Self::Error>>;
}

pub trait RendererFactory {
    type Renderer: Renderer;
    type Error;

    fn create(
        &self,
        fetcher: SharedClientFactory,
    ) -> impl Future<Output = Result<Self::Renderer, Self::Error>>;
}

impl Renderer for () {
    type Error = Infallible;
    fn render<'a>(
        &'a self,
        _path: RelativePathBuf,
        _req: Request<Body>,
    ) -> BoxFuture<'a, Result<RenderResult, Self::Error>> {
        Box::pin(async move {
            Ok(RenderResult {
                content: Bytes::new(),
                assets: Default::default(),
                head: Default::default(),
            })
        })
    }
}

impl<T> Renderer for Arc<T>
where
    T: Renderer + Send + Sync,
{
    type Error = T::Error;
    fn render<'a>(
        &'a self,
        path: RelativePathBuf,
        req: Request<Body>,
    ) -> BoxFuture<'a, Result<RenderResult, Self::Error>> {
        (**self).render(path, req)
    }
}

impl<T> Renderer for Option<T>
where
    T: Renderer + Send + Sync,
{
    type Error = T::Error;
    fn render<'a>(
        &'a self,
        path: RelativePathBuf,
        req: Request<Body>,
    ) -> BoxFuture<'a, Result<RenderResult, Self::Error>> {
        Box::pin(async move {
            match self {
                Some(ret) => ret.render(path, req).await,
                None => Ok(RenderResult {
                    content: Bytes::new(),
                    assets: Default::default(),
                    head: Default::default(),
                }),
            }
        })
    }
}
