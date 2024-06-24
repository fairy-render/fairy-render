#[derive(Debug, thiserror::Error)]
pub enum ViteError {
    #[error("render error: {0}")]
    Render(Box<dyn std::error::Error + Send + Sync>),
    #[error("manifest not found: {path} or errored: {error:?}")]
    Manifest {
        path: String,
        error: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
