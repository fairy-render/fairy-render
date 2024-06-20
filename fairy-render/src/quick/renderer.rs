use core::fmt;
use std::{path::PathBuf, pin::Pin, sync::Arc};

use futures_core::Future;
use klaver_http::set_client_box;
use klaver_module::Modules;
use klaver_worker::{ModuleId, Persistence, Worker};
use reggie::SharedClientFactory;
use relative_path::RelativePathBuf;
use rquickjs::{Ctx, FromJs, Function, Module, Value};

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
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let Ok(obj) = value.try_into_object() else {
            return Err(rquickjs::Error::new_from_js("value", "object"));
        };

        Ok(JsResult {
            content: obj.get("content")?,
            files: obj.get("files")?,
            head: obj.get("head")?,
        })
    }
}

fn new_worker(
    client: SharedClientFactory,
    search_paths: Vec<PathBuf>,
) -> klaver_worker::pool::Pool {
    let pool =
        klaver_worker::pool::Pool::builder(klaver_worker::pool::Manager::new_with_customize(
            move |runtime, ctx| {
                let search_paths = search_paths.clone();
                Box::pin(async move {
                    let mut modules = Modules::default();
                    for path in &search_paths {
                        modules.add_search_path(path);
                    }

                    modules.add_search_path(".");

                    modules.register::<klaver_base::Module>("@klaver/base");
                    modules.register::<klaver_http::Module>("@klaver/http");

                    modules.attach(runtime).await;

                    Ok(())
                })
            },
            move |ctx, _| {
                let client = client.clone();
                Box::pin(async move {
                    ctx.globals().set(
                        "print",
                        Function::new(ctx.clone(), |arg: rquickjs::Value| {
                            println!("{}", arg.try_into_string().unwrap().to_string()?);
                            rquickjs::Result::Ok(())
                        }),
                    )?;

                    set_client_box(&ctx, client)?;

                    klaver_compat::init(&ctx)?;

                    ctx.eval(GLOBALS)?;

                    Ok(())
                })
            },
        ))
        .max_size(10)
        .build()
        .unwrap();

    pool
}

#[derive(Clone)]
pub struct Quick {
    worker: klaver_worker::pool::Pool,
}

impl Quick {
    pub fn new(client: SharedClientFactory, search_paths: Vec<PathBuf>) -> Quick {
        Quick {
            worker: new_worker(client, search_paths),
        }
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
    Engine(klaver_worker::Error),
    Pool(klaver_worker::pool::PoolError),
    Script(ScriptError),
}

impl From<klaver_worker::Error> for QuickRenderError {
    fn from(value: klaver_worker::Error) -> Self {
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

    type Error = klaver_worker::Error;

    fn create(
        &self,
        fetcher: reggie::SharedClientFactory,
    ) -> impl Future<Output = Result<Self::Renderer, Self::Error>> {
        async move { Ok(Quick::new(fetcher, self.search_paths.clone())) }
    }
}

impl Renderer for Quick {
    type Error = QuickRenderError;

    fn render<'a>(
        &'a self,
        path: RelativePathBuf,
        req: reggie::http::Request<reggie::Body>,
    ) -> futures_core::future::BoxFuture<'a, Result<crate::renderer::RenderResult, Self::Error>>
    {
        Box::pin(async move {
            let worker = self.worker.get().await.map_err(QuickRenderError::Pool)?;

            let ret = klaver_worker::async_with!(worker => |ctx, _p| {

                let fairy:  rquickjs::Object = ctx.globals().get("Fairy")?;
                let run_main: rquickjs::Function = fairy.get("runMain")?;

                let req = klaver_http::Request::from_request(&ctx, req)?;

                let ret = run_main.call::<_,rquickjs::Promise>((path.as_str(), req,))?;
                let ret = ret.into_future::<JsResult>().await?;

                Ok(RenderResult {
                    content: ret.content.into(),
                    assets: ret.files,
                    head: ret.head
                })
            })
            .await;

            let ret = match ret {
                Ok(ret) => ret,
                Err(klaver_worker::Error::Script(rquickjs::Error::Exception)) => {
                    let (message, stack, file, line, column) = worker
                        .with(|ctx| {
                            let err = ctx.catch();

                            if let Some(exp) = err.into_exception() {
                                Ok((
                                    exp.message(),
                                    exp.stack(),
                                    exp.file(),
                                    exp.line(),
                                    exp.column(),
                                ))
                            } else {
                                Ok((None, None, None, None, None))
                            }
                        })
                        .await?;

                    return Err(QuickRenderError::Script(ScriptError {
                        message,
                        stack,
                        file,
                        line,
                        column,
                    }));
                }
                Err(err) => return Err(err.into()),
            };

            Ok(ret)
        })
    }
}
