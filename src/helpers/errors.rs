use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdbirError {
    #[error("serde_yaml error\n{0}")]
    SerdeYamlError(#[from] serde_yaml::Error),

    #[error("IO error\n{0}")]
    IOError(#[from] std::io::Error),
}
