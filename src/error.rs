use std::error::Error as StdError;
use std::fmt;
use ipfs_api::response::Error as IpfsError;
use substrate_subxt::Error as SubXtError;
use config::ConfigError;

#[derive(Debug)]
pub enum MinerErrorKind {
    FileNotFound,
    CallError,
    Msg(String),
    Io(::std::io::Error),
    Ipfs(IpfsError),
    Config(ConfigError),
    SubXt(SubXtError),
}


/// The Error type
#[derive(Debug)]
pub struct MinerError {
    /// Kind of error
    pub kind: MinerErrorKind,
    pub source: Option<Box<dyn StdError + Send + Sync>>,
}


impl StdError for MinerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.source {
            Some(ref err) => Some(&**err),
            _ => None,
        }
    }
}

impl fmt::Display for MinerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            // TODO: other
            MinerErrorKind::Msg(ref message) => write!(f, "{}", message),
            MinerErrorKind::Io(ref e) => write!(f, "{}", e),
            MinerErrorKind::Ipfs(ref e) => write!(f, "connection error"),
            MinerErrorKind::Config(ref e) => write!(f, "Config error"),
            MinerErrorKind::SubXt(ref e) => write!(f, "connection error"),
            MinerErrorKind::FileNotFound => write!(f, "The accessed file does not exist"),
            MinerErrorKind::CallError => write!(f, "The requested method does not exist"),
        }
    }
}


impl MinerError {
    pub fn msg(value: impl ToString) -> Self {
        Self { kind: MinerErrorKind::Msg(value.to_string()), source: None }
    }
}


impl From<&str> for MinerError {
    fn from(e: &str) -> Self {
        Self::msg(e)
    }
}

impl From<String> for MinerError {
    fn from(e: String) -> Self {
        Self::msg(e)
    }
}

impl From<IpfsError> for MinerError {
    fn from(e: IpfsError) -> Self {
        Self { kind: MinerErrorKind::Ipfs(e), source: None }
    }
}

impl From<ConfigError> for MinerError {
    fn from(e: ConfigError) -> Self {
        Self { kind: MinerErrorKind::Config(e), source: None }
    }
}

impl From<SubXtError> for MinerError {
    fn from(e: SubXtError) -> Self {
        Self { kind: MinerErrorKind::SubXt(e), source: None }
    }
}


impl From<::std::io::Error> for MinerError {
    fn from(e: ::std::io::Error) -> Self {
        Self { kind: MinerErrorKind::Io(e), source: None }
    }
}


pub type Result<T> = ::std::result::Result<T, MinerError>;