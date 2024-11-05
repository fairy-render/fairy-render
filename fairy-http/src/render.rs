use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use axum::http::Uri;
use fairy_render::quick::Quick;
use fairy_vite::{FairyRenderer, Vite, ViteEntry};
use reggie::bytes::Bytes;
use reggie::http::{Request, Response};
use reggie::http_body::Body as HttpBody;
use reggie::http_body_util::BodyExt;
use reggie::Body;
use tower_service::Service;

use crate::template::Template;

#[derive(Clone)]
pub struct RenderService {
    fairy: FairyRenderer,
    template: Arc<dyn Template + Send + Sync>,
}

impl RenderService {
    pub fn new<T>(fairy: FairyRenderer, func: T) -> RenderService
    where
        T: Template + Send + Sync + 'static,
    {
        RenderService {
            fairy,
            template: Arc::new(func),
        }
    }
}

impl<B> Service<Request<B>> for RenderService
where
    B: HttpBody + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
    B::Data: Into<Bytes>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let quick = self.fairy.clone();
        let template = self.template.clone();
        Box::pin(async move {
            let uri = req.uri().clone();

            let result = quick
                .render(req.map(|m| {
                    reggie::Body::from_streaming(
                        m.map_err(|err| reggie::Error::Body(Box::new(err))),
                    )
                }))
                .await;

            let output = template.render(uri, result);

            let resp = Response::builder()
                .header("Content-Type", "text/html")
                .status(200)
                .body(Body::from(output))
                .expect("build response");

            Ok(resp)
        })
    }
}

#[derive(Clone)]
pub struct FairyRenderService {
    fairy: FairyRenderer,
    template: Arc<dyn Template + Send + Sync>,
}

impl FairyRenderService {
    pub fn new<T>(fairy: FairyRenderer, func: T) -> FairyRenderService
    where
        T: Template + Send + Sync + 'static,
    {
        FairyRenderService {
            fairy,
            template: Arc::new(func),
        }
    }
}

impl<B> Service<Request<B>> for FairyRenderService
where
    B: HttpBody + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
    B::Data: Into<Bytes>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let quick = self.fairy.clone();
        let template = self.template.clone();
        Box::pin(async move {
            let uri = req.uri().clone();

            if req.uri().scheme().is_none() {
                *req.uri_mut() = format!("internal://internal.com{}", uri)
                    .parse()
                    .expect("url");
            }

            let result = quick
                .render(req.map(|m| {
                    reggie::Body::from_streaming(
                        m.map_err(|err| reggie::Error::Body(Box::new(err))),
                    )
                }))
                .await;

            let output = template.render(uri, result);

            let resp = Response::builder()
                .header("Content-Type", "text/html")
                .status(200)
                .body(Body::from(output))
                .expect("build response");

            Ok(resp)
        })
    }
}
