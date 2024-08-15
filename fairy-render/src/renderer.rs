use std::{convert::Infallible, sync::Arc};

use futures::{future::BoxFuture, Future};
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
    type Future<'a>: Future<Output = Result<RenderResult, Self::Error>>
    where
        Self: 'a;
    fn render<'a>(&'a self, path: RelativePathBuf, req: Request<Body>) -> Self::Future<'a>;
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
    type Future<'a> = core::future::Ready<Result<RenderResult, Self::Error>>;
    fn render<'a>(&'a self, _path: RelativePathBuf, _req: Request<Body>) -> Self::Future<'a> {
        core::future::ready(Ok(RenderResult {
            content: Bytes::new(),
            assets: Default::default(),
            head: Default::default(),
        }))
    }
}

impl<T> Renderer for Arc<T>
where
    T: Renderer + Send + Sync,
    for<'a> T: 'a,
{
    type Error = T::Error;
    type Future<'a> = T::Future<'a>;
    fn render<'a>(&'a self, path: RelativePathBuf, req: Request<Body>) -> Self::Future<'a> {
        (**self).render(path, req)
    }
}

impl<T> Renderer for Option<T>
where
    T: Renderer + Send + Sync,
    for<'a> T: 'a,
{
    type Error = T::Error;
    type Future<'a> = futures::future::Either<
        T::Future<'a>,
        core::future::Ready<Result<RenderResult, Self::Error>>,
    >;
    fn render<'a>(&'a self, path: RelativePathBuf, req: Request<Body>) -> Self::Future<'a> {
        match self {
            Some(ret) => futures::future::Either::Left(ret.render(path, req)),
            None => futures::future::Either::Right(core::future::ready(Ok(RenderResult {
                content: Bytes::new(),
                assets: Default::default(),
                head: Default::default(),
            }))),
        }
    }
}
