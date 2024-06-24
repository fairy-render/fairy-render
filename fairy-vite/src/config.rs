use crate::{util::load_json, ViteError};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViteConfig {
    pub assets: String,
    pub assets_path: String,
    pub base: String,
    pub client_manifest: String,
    pub entries: EntryValue,
    pub port: u16,
    pub root: String,
    pub server_manifest: String,
    pub ssr_manifest: String,
    pub work_dir: String,
}

impl ViteConfig {
    pub fn get_entry(&self, name: Option<&str>) -> Option<&Entry> {
        match (name, &self.entries) {
            (Some(name), EntryValue::Many(map)) => map.get(name),
            (None, EntryValue::Entry(entry)) => Some(entry),
            _ => None,
        }
    }

    pub fn work_dir(&self) -> &Path {
        Path::new(&self.work_dir)
    }

    pub fn assets(&self) -> PathBuf {
        self.work_dir().join(&self.assets)
    }

    /// The build root
    pub fn root(&self) -> PathBuf {
        self.work_dir().join(&self.root)
    }

    // pub fn build_dev<T: Template + Send + Sync + Clone + 'static, B>(
    //     self,
    //     template: T,
    //     entry: RouteMap<'_>,
    // ) -> Result<ViteService<B>, ViteError> {
    //     let mut router = Router::<()>::new();

    //     if entry.map.is_empty() {
    //         router = router.fallback_service(ViteDevService::new(self, template, None));
    //     } else {
    //         for (entry, route) in entry.map {
    //             let Some(entry) = self.get_entry(Some(entry)) else {
    //                 panic!("entry not found")
    //             };

    //             router = router.nest_service(
    //                 &format!("{route}"),
    //                 ViteDevService::new(self.clone(), template.clone(), &*entry.client),
    //             );
    //         }
    //     }

    //     Ok(ViteService {
    //         inner: router.into_service(),
    //     })
    // }

    // pub async fn build<T: Template + Send + Sync + Clone + 'static, F, B>(
    //     self,
    //     template: T,
    //     http_factory: F,
    //     entry: RouteMap<'_>,
    // ) -> ViteService<B>
    // where
    //     F: HttpClientFactory + Send + Sync + Clone + 'static,
    //     F::Client<Body>: Send + Sync + 'static,
    //     <F::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    //     for<'b> <F::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
    // {
    //     let mut router = self
    //         .build_router::<_, F>(entry, template, http_factory)
    //         .await
    //         .unwrap();
    //     router = router.nest_service(&self.assets_path, ServeDir::new(self.assets()));

    //     ViteService {
    //         inner: router.into_service(),
    //     }
    // }

    // async fn build_entry<F>(
    //     &self,
    //     factory: QuickFactory,
    //     http_factory: F,
    //     entry: &Entry,
    // ) -> Result<Vite<Quick>, ViteError>
    // where
    //     F: HttpClientFactory + Send + Sync + 'static,
    //     F::Client<Body>: Send + Sync + 'static,
    //     <F::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    //     for<'b> <F::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
    // {
    //     let config = ViteOptions {
    //         client: ClientEntry::new(&entry.client)
    //             .manifest(&self.client_manifest)
    //             .output("")
    //             .ssr_manifest(&self.ssr_manifest),
    //         server: ServerEntry::new(&entry.server)
    //             .output("")
    //             .manifest(&self.server_manifest),
    //         path: Path::new(&self.work_dir).join(&self.root),
    //     }
    //     .build_with::<_, F>(factory, http_factory)
    //     .await?;

    //     Ok(config)
    // }

    // async fn build_router<T, F: HttpClientFactory + Send + Sync + 'static>(
    //     &self,
    //     entry: RouteMap<'_>,
    //     template: T,
    //     http_factory: F,
    // ) -> Result<Router, ViteError>
    // where
    //     T: Template + Send + Sync + Clone + 'static,
    //     F: HttpClientFactory + Send + Sync + Clone + 'static,
    //     F::Client<Body>: Send + Sync + 'static,
    //     for<'b> <F::Client<Body> as HttpClient<Body>>::Future<'b>: Send,
    //     <F::Client<Body> as HttpClient<Body>>::Body: Into<reggie::Body>,
    // {
    //     let mut router = Router::<()>::new();

    //     let dist = self.root();
    //     let mut factory = QuickFactory::default();
    //     factory.add_search_path(dist.display().to_string());

    //     if entry.map.is_empty() {
    //         let Some(entry) = self.get_entry(None) else {
    //             panic!("entry not found")
    //         };

    //         let vite = self.build_entry::<F>(factory, http_factory, entry).await?;

    //         router = router.fallback_service(RenderService::new(
    //             vite,
    //             ViteEntry {
    //                 server: entry.server.clone(),
    //                 client: entry.client.clone().into(),
    //             },
    //             template,
    //         ));
    //     } else {
    //         for (entry, route) in entry.map {
    //             let Some(entry) = self.get_entry(Some(entry)) else {
    //                 panic!("entry not found")
    //             };

    //             let vite = self
    //                 .build_entry::<F>(factory.clone(), http_factory.clone(), entry)
    //                 .await?;

    //             router = router.route_service(
    //                 &format!("{route}"),
    //                 RenderService::new(
    //                     vite,
    //                     ViteEntry {
    //                         client: entry.client.clone().into(),
    //                         server: entry.server.clone(),
    //                     },
    //                     template.clone(),
    //                 ),
    //             );
    //         }
    //     }

    //     Ok(router)
    // }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum EntryValue {
    Entry(Entry),
    Many(HashMap<String, Entry>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub client: String,
    pub server: String,
}

impl ViteConfig {
    pub async fn load(path: &Path) -> Result<ViteConfig, ViteError> {
        load_json(path).await
    }
}
