use crate::{render::FairyRenderService, Template, ViteDevService, ViteService};
use axum::Router;
use fairy_render::quick::QuickFactory;
use fairy_vite::{Fairy, ViteConfig, ViteError};
use reggie::{Body, HttpClient, HttpClientFactory};
use std::collections::HashMap;
use std::future::Future;
use tower_http::services::ServeDir;

#[derive(Debug, Default)]
pub struct RouteMap<'a> {
    map: HashMap<&'a str, &'a str>,
}

impl<'a> RouteMap<'a> {
    pub fn add(&mut self, entry: &'a str, route: &'a str) -> &mut Self {
        self.map.insert(entry, route);
        self
    }

    pub fn map(mut self, entry: &'a str, route: &'a str) -> Self {
        self.map.insert(entry, route);
        self
    }
}

pub fn build_dev<T: Template + Send + Sync + Clone + 'static, B>(
    fairy: Fairy,
    template: T,
    entry: RouteMap<'_>,
) -> Result<ViteService<B>, ViteError> {
    let mut router = Router::<()>::new();

    if entry.map.is_empty() {
        router = router.fallback_service(FairyRenderService::new(
            fairy.create_renderer(None),
            template,
        ));
    } else {
        for (entry, route) in entry.map {
            let Some(entry) = fairy.config().get_entry(Some(entry)) else {
                panic!("entry not found")
            };

            router = router.nest_service(
                &format!("{route}"),
                ViteDevService::new(fairy.config().clone(), template.clone(), &*entry.client),
            );
        }
    }

    Ok(ViteService {
        inner: router.into_service(),
    })
}

pub async fn build<T: Template + Send + Sync + Clone + 'static, B>(
    fairy: Fairy,
    template: T,
    entry: RouteMap<'_>,
) -> ViteService<B> {
    let mut router = build_router(&fairy, entry, template).await.unwrap();
    router = router.nest_service(
        &fairy.config().assets_path,
        ServeDir::new(fairy.config().assets()),
    );

    ViteService {
        inner: router.into_service(),
    }
}

async fn build_router<T>(
    fairy: &Fairy,
    entry: RouteMap<'_>,
    template: T,
) -> Result<Router, ViteError>
where
    T: Template + Send + Sync + Clone + 'static,
{
    let mut router = Router::<()>::new();

    let dist = fairy.config().root();
    let mut factory = QuickFactory::default();
    factory.add_search_path(dist.display().to_string());

    if entry.map.is_empty() {
        let Some(entry) = fairy.config().get_entry(None) else {
            panic!("entry not found")
        };

        router = router.fallback_service(FairyRenderService::new(
            fairy.create_renderer(None),
            template,
        ));
    } else {
        for (entry, route) in entry.map {
            router = router.route_service(
                &format!("{route}"),
                FairyRenderService::new(fairy.create_renderer(entry), template.clone()),
            );
        }
    }

    Ok(router)
}

pub trait ViteConfigExt {
    fn build<T: Template + Send + Sync + Clone + 'static, B, H>(
        self,
        http: H,
        template: T,
        routes: RouteMap<'_>,
    ) -> impl Future<Output = Result<ViteService<B>, ViteError>> + Send
    where
        H: HttpClientFactory + Send + Sync + 'static,
        H::Client<Body>: Send + Sync + 'static,
        for<'b> <H::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
        <H::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>;

    fn build_dev<T: Template + Send + Sync + Clone + 'static, B>(
        self,
        template: T,
        entry: RouteMap<'_>,
    ) -> impl Future<Output = Result<ViteService<B>, ViteError>> + Send;
}

impl ViteConfigExt for ViteConfig {
    fn build<T: Template + Send + Sync + Clone + 'static, B, H>(
        self,
        http: H,
        template: T,
        routes: RouteMap<'_>,
    ) -> impl Future<Output = Result<ViteService<B>, ViteError>> + Send
    where
        H: HttpClientFactory + Send + Sync + 'static,
        H::Client<Body>: Send + Sync + 'static,
        for<'b> <H::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
        <H::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    {
        async move {
            let fairy = Fairy::new(self, http).await?;
            Ok(build(fairy, template, routes).await)
        }
    }

    fn build_dev<T: Template + Send + Sync + Clone + 'static, B>(
        self,
        template: T,
        routes: RouteMap<'_>,
    ) -> impl Future<Output = Result<ViteService<B>, ViteError>> + Send {
        async move {
            let fairy = Fairy::dev(self)?;
            Ok(build(fairy, template, routes).await)
        }
    }
}
