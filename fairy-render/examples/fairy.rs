use std::path::PathBuf;

use fairy_render::{quick::Quick, Renderer};
use reggie::{http::Request, Body, Reqwest};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt().init();

    let quick = Quick::new(
        reggie::factory_arc(Reqwest::default()),
        vec![PathBuf::from(".")],
    );

    let ret = quick
        .render(
            "./packages/solid-example/dist/server/entry-server.js".into(),
            Request::builder()
                .uri("http://internal/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    println!("{:?}", ret);
}
