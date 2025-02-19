use std::path::Path;

use axum::{http::Uri, routing::get, Router};
use fairy_http::config::ViteConfigExt;
use fairy_http::{config::RouteMap, Template};
use fairy_render::reggie::{Body, HttpClient, HttpClientFactory};
use fairy_vite::ViteConfig;
use fairy_vite::{AssetKind, FairyResult, ViteError};
use futures::future::BoxFuture;

markup::define! {
    Home(req: FairyResult) {
        @markup::doctype()
        html {
            head {
                title { "Hello" }
                style {
                    "body { background: #fafbfc; }"
                    "#main { padding: 2rem; }"
                }
                @for file in &req.assets {
                    @match file.kind {
                        AssetKind::Styling => {
                            link[href= format!("{}", &file.file), rel="stylesheet", class="en klasse"] {  }
                        }
                        _ => {
                            ""
                        }
                    }
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
                            script[src= format!("{}", &file.file), type="module"] {  }
                        }
                        _ => {
                            ""
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
        request: Result<fairy_vite::FairyResult, fairy_vite::ViteError>,
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
    // tracing_subscriber::fmt()
    //     .with_max_level(LevelFilter::TRACE)
    //     .init();

    let mut app = Router::new();

    let solid_config = ViteConfig::load(Path::new("solid-config.json"))
        .await
        .unwrap();

    let fetcher = Fetcher {
        client: reqwest::Client::new(),
    };

    let solid = solid_config
        .build(fetcher.clone(), T, RouteMap::default())
        .await
        .unwrap();

    app = app.route("/api/message", get(api)).fallback_service(solid);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone)]
pub struct Fetcher {
    client: reqwest::Client,
}

impl HttpClientFactory for Fetcher {
    fn create<B>(&self) -> Self::Client<B>
    where
        B: fairy_render::reggie::http_body::Body + Send + 'static,
        B::Data: Into<axum::body::Bytes> + Send,
        B::Error: Into<fairy_render::reggie::Error> + Send,
    {
        self.clone()
    }

    type Client<B> = Self
    where
        B: fairy_render::reggie::http_body::Body + Send + 'static,
        B::Data: Into<axum::body::Bytes> + Send,
        B::Error: Into<fairy_render::reggie::Error> + Send;
}

impl<B> HttpClient<B> for Fetcher
where
    B: fairy_render::reggie::http_body::Body + Send + 'static,
    B::Data: Into<axum::body::Bytes> + Send,
    B::Error: Into<fairy_render::reggie::Error> + Send,
{
    type Body = Body;

    type Future<'a> =
        BoxFuture<'a, Result<axum::http::Response<Self::Body>, fairy_render::reggie::Error>>;

    fn send<'a>(
        &'a self,
        request: axum::http::Request<B>,
    ) -> BoxFuture<'a, Result<axum::http::Response<Self::Body>, fairy_render::reggie::Error>> {
        Box::pin(async move {
            if request.uri().scheme_str() == Some("internal") {
                return Ok(fairy_render::reggie::http::Response::builder()
                    .body(Body::from(String::from("Hello, World!")))
                    .unwrap());
            }

            let resp = self.client.send(request).await?;

            Ok(resp.map(|m| Body::from_streaming(m)))
        })
    }
}
