use std::path::Path;

use crate::error::ViteError;

pub async fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ViteError> {
    let cmb: Vec<u8> = tokio::fs::read(path)
        .await
        .map_err(|err| ViteError::Manifest {
            path: path.display().to_string(),
            error: Some(Box::new(err)),
        })?;

    serde_json::from_slice(&cmb).map_err(|err| ViteError::Manifest {
        path: path.display().to_string(),
        error: Some(Box::new(err)),
    })
}
