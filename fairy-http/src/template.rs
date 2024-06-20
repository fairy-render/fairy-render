use std::sync::Arc;

use axum::http::Uri;
use fairy_render::{vite::ViteError, FairyResult};

pub trait Template {
    fn render(&self, uri: Uri, request: Result<FairyResult, ViteError>) -> String;
}

impl Template for Arc<dyn Template + Send + Sync> {
    fn render(&self, uri: Uri, request: Result<FairyResult, ViteError>) -> String {
        (**self).render(uri, request)
    }
}
