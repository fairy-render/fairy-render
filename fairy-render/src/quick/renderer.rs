use core::fmt;
use std::path::PathBuf;

use futures::{future::BoxFuture, Future};
use klaver::{
    pool::Pool,
    quick::{self, CatchResultExt, Class, Ctx, FromJs, Object},
    vm::VmOptions,
};
use klaver_http::set_client_box;
use reggie::SharedClientFactory;
use relative_path::{RelativePath, RelativePathBuf};

use crate::{
    renderer::{RenderResult, Renderer},
    RendererFactory,
};

const GLOBALS: &[u8] = include_bytes!("globals.js");

struct JsResult {
    content: String,
    files: Vec<String>,
    head: Vec<String>,
}

impl<'js> FromJs<'js> for JsResult {
    fn from_js(ctx: &Ctx<'js>, value: quick::Value<'js>) -> quick::Result<Self> {
        let Ok(obj) = value.try_into_object() else {
            return Err(quick::Error::new_from_js("value", "object"));
        };

        Ok(JsResult {
            content: obj.get("content")?,
            files: obj.get("files")?,
            head: obj.get("head")?,
        })
    }
}

#[derive(Clone)]
pub struct Quick {
    worker: Pool,
}

impl Quick {
    pub fn new(client: SharedClientFactory, search_paths: Vec<PathBuf>) -> Quick {
        let mut opts = VmOptions::default().module::<klaver_compat::Compat>();

        for sp in search_paths {
            opts = opts.search_path(sp);
        }

        let pool = Pool::builder(klaver::pool::Manager::new(opts).init(move |vm| {
            let client = client.clone();
            Box::pin(async move {
                vm.run_with(|ctx| {
                    set_client_box(ctx, client)?;
                    Ok(())
                })
                .await?;

                klaver_compat::init(vm).await?;

                vm.run_with(|ctx| {
                    ctx.eval(GLOBALS)?;
                    Ok(())
                })
                .await?;

                Ok(())
            })
        }))
        .build()
        .unwrap();

        Quick { worker: pool }
    }
}

#[derive(Debug)]
pub struct ScriptError {
    message: Option<String>,
    stack: Option<String>,
    file: Option<String>,
    line: Option<i32>,
    column: Option<i32>,
}

impl ScriptError {
    /// Returns the message of the error.
    ///
    /// Same as retrieving `error.message` in JavaScript.
    pub fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    /// Returns the file name from with the error originated..
    ///
    /// Same as retrieving `error.fileName` in JavaScript.
    pub fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }

    /// Returns the file line from with the error originated..
    ///
    /// Same as retrieving `error.lineNumber` in JavaScript.
    pub fn line(&self) -> Option<i32> {
        self.line
    }

    /// Returns the file line from with the error originated..
    ///
    /// Same as retrieving `error.lineNumber` in JavaScript.
    pub fn column(&self) -> Option<i32> {
        self.column
    }

    /// Returns the error stack.
    ///
    /// Same as retrieving `error.stack` in JavaScript.
    pub fn stack(&self) -> Option<&String> {
        self.stack.as_ref()
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Error:".fmt(f)?;
        let mut has_file = false;
        if let Some(file) = &self.file {
            '['.fmt(f)?;
            file.fmt(f)?;
            ']'.fmt(f)?;
            has_file = true;
        }
        if let Some(line) = &self.line {
            if *line > -1 {
                if has_file {
                    ':'.fmt(f)?;
                }
                line.fmt(f)?;
            }
        }
        if let Some(column) = &self.column {
            if *column > -1 {
                ':'.fmt(f)?;
                column.fmt(f)?;
            }
        }
        if let Some(message) = &self.message {
            ' '.fmt(f)?;
            message.fmt(f)?;
        }
        if let Some(stack) = &self.stack {
            '\n'.fmt(f)?;
            stack.fmt(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for ScriptError {}

#[derive(Debug)]
pub enum QuickRenderError {
    Engine(klaver::Error),
    Pool(klaver::pool::PoolError),
    Script(ScriptError),
}

impl From<klaver::Error> for QuickRenderError {
    fn from(value: klaver::Error) -> Self {
        QuickRenderError::Engine(value)
    }
}

impl fmt::Display for QuickRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Engine(e) => write!(f, "{e}"),
            Self::Script(e) => write!(f, "{e}"),
            Self::Pool(e) => write!(f, "{e}",),
        }
    }
}

impl std::error::Error for QuickRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(e) => Some(e),
            Self::Pool(e) => Some(e),
            Self::Script(e) => Some(e),
        }
    }
}

#[derive(Default, Clone)]
pub struct QuickFactory {
    search_paths: Vec<PathBuf>,
}

impl QuickFactory {
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.search_paths.push(path.into());
        self
    }

    pub fn search_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }
}

impl RendererFactory for QuickFactory {
    type Renderer = Quick;

    type Error = klaver::Error;

    fn create(
        &self,
        fetcher: reggie::SharedClientFactory,
    ) -> impl Future<Output = Result<Self::Renderer, Self::Error>> {
        async move { Ok(Quick::new(fetcher, self.search_paths.clone())) }
    }
}

impl Renderer for Quick {
    type Error = QuickRenderError;

    type Future<'a> = BoxFuture<'a, Result<RenderResult, Self::Error>>;

    fn render<'a>(
        &'a self,
        path: RelativePathBuf,
        req: reggie::http::Request<reggie::Body>,
    ) -> BoxFuture<'a, Result<crate::renderer::RenderResult, Self::Error>> {
        Box::pin(async move {
            let worker = self.worker.get().await.map_err(QuickRenderError::Pool)?;

            let ret = klaver::async_with!(worker => |ctx| {

                let req = klaver_http::Request::from_request(&ctx, req).catch(&ctx)?;
                Ok(render(&ctx, &path, req).await.catch(&ctx)?)


            })
            .await?;

            Ok(ret)
        })
    }
}

pub async fn render<'js>(
    ctx: &Ctx<'js>,
    path: &RelativePath,
    req: Class<'js, klaver_http::Request<'js>>,
) -> klaver::quick::Result<RenderResult> {
    let globals = ctx.globals();
    if !globals.contains_key("Fairy")? {
        ctx.eval(GLOBALS)?;
    }

    let fairy: Object = globals.get("Fairy")?;
    let run_main: quick::Function = fairy.get("runMain")?;

    let ret = run_main.call::<_, quick::Promise>((path.as_str(), req))?;
    let ret = ret.into_future::<JsResult>().await?;

    Ok(RenderResult {
        content: ret.content.into(),
        assets: ret.files,
        head: ret.head,
    })
}
