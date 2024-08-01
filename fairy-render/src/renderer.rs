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
