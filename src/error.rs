use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, PinyinSortError>;

#[derive(Debug)]
pub enum PinyinSortError {
    InvalidArgument(String),
    Io {
        context: String,
        source: std::io::Error,
    },
    OverrideParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidOverride(String),
}

impl PinyinSortError {
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

impl Display for PinyinSortError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "{message}"),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::OverrideParse { path, source } => {
                write!(
                    f,
                    "failed to parse override config {}: {source}",
                    path.display()
                )
            }
            Self::InvalidOverride(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PinyinSortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::OverrideParse { source, .. } => Some(source),
            Self::InvalidArgument(_) | Self::InvalidOverride(_) => None,
        }
    }
}
