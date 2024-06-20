use std::path::{Path, PathBuf};

use axum::{http::Uri, response::Html, routing::get, Router};
use fairy_http::{
    config::{RouteMap, ViteConfig},
    Template,
};
use fairy_render::{
    quick::{Quick, QuickFactory},
    reggie::{Body, HttpClient, HttpClientFactory, Reqwest},
    vite::{ClientEntry, ServerEntry, ViteError, ViteOptions},
    AssetKind,
};
use futures::future::BoxFuture;

markup::define! {
    Home(req: fairy_render::FairyResult) {
        @markup::doctype()
        html {
            head {
                title { "Hello" }
                style {
                    "body { background: #fafbfc; }"
                    "#main { padding: 2rem; }"
                }
                @for head in &req.head {
                    @markup::raw(head)
                }

            }
            body {
                #root {
                    @{
                        markup::raw(std::str::from_utf8(&req.content).unwrap())
                    }
                }
                @for file in &req.assets {
                    @match file.kind {
                        AssetKind::Script => {
                            script[src= &file.file, type="module"] {  }
                        }
                        _ => {
                            "Not"
                        }
                    }
                }
            }
        }
    }

}

markup::define! {
    Fail(req: ViteError, uri: Uri) {
        @markup::doctype()
        html {
            head {
                title { "Hello" }
                style {
                    "body { background: #fafbfc; }"
                    "#main { padding: 2rem; }"
                }


            }
            body {
                h1 {
                    "Render failed"
                }
                p {
                    "The url "
                    @uri.to_string()
                    " Failed"
                }
                pre {
                    @req.to_string()
                }
            }
        }
    }

}

#[derive(Debug, Clone, Copy)]
struct T;

impl Template for T {
    fn render(
        &self,
        uri: axum::http::Uri,
        request: Result<fairy_render::FairyResult, fairy_render::vite::ViteError>,
    ) -> String {
        match request {
            Ok(request) => Home { req: request }.to_string(),
            Err(err) => Fail { req: err, uri }.to_string(),
        }
    }
}

async fn api() -> &'static str {
    "Hello, World!"
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut app = Router::new();

    let vite_config = ViteConfig::load(Path::new("vite-config.json"))
        .await
        .unwrap();

    let service = vite_config
        .build(
            T,
            Fetcher {
                client: fairy_render::reqwest::Client::new(),
            },
            RouteMap::default(),
        )
        .await;

    // let service = vite_config.build_dev(T, RouteMap::default()).unwrap();

    app = app
        .route("/api/message", get(api))
        // .nest_service("/assets", assets)
        .fallback_service(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone)]
pub struct Fetcher {
    client: fairy_render::reqwest::Client,
}

impl HttpClientFactory for Fetcher {
    fn create<B>(&self) -> Self::Client<B>
    where
        B: fairy_render::reggie::http_body::Body + Send + 'static,
        B::Data: Into<axum::body::Bytes>,
        B::Error: Into<fairy_render::reggie::Error>,
    {
        self.clone()
    }

    type Client<B> = Self
    where
        B: fairy_render::reggie::http_body::Body + Send + 'static,
        B::Data: Into<axum::body::Bytes>,
        B::Error: Into<fairy_render::reggie::Error>;
}

impl<B> HttpClient<B> for Fetcher
where
    B: fairy_render::reggie::http_body::Body + Send + 'static,
    B::Data: Into<axum::body::Bytes>,
    B::Error: Into<fairy_render::reggie::Error>,
{
    type Body = Body;

    type Future<'a> =
        BoxFuture<'a, Result<axum::http::Response<Self::Body>, fairy_render::reggie::Error>>;

    fn send<'a>(
        &'a self,
        request: axum::http::Request<B>,
    ) -> BoxFuture<'a, Result<axum::http::Response<Self::Body>, fairy_render::reggie::Error>> {
        Box::pin(async move {
            if request.uri().scheme().is_none() {
                return Ok(fairy_render::reggie::http::Response::builder()
                    .body(Body::from(String::from("Hello, World!")))
                    .unwrap());
            }

            let resp = self.client.send(request).await?;

            Ok(resp.map(|m| Body::from_streaming(m)))
        })
    }
}
