use std::error::Error as StdError;
use std::fmt;
use ipfs_api::response::Error as IpfsError;
use substrate_subxt::Error as SubXtError;
use config::ConfigError;
use reqwest::Error as ReqwestError;
use hex::FromHexError;
use sp_core::crypto::{SecretStringError, PublicError};
use codec::Error as CodecError;
use std::option::NoneError;
use rocket::config::ConfigError as RocketConfigError;

#[derive(Debug)]
pub enum MinerErrorKind {
    FileNotFound,
    CallError,
    Msg(String),
    Io(::std::io::Error),
    Ipfs(IpfsError),
    Config(ConfigError),
    SubXt(SubXtError),
    Reqwest(ReqwestError),
    Hex(FromHexError),
    SecretString(SecretStringError),
    PublicError(PublicError),
    Codec(CodecError),
    None(NoneError),
    RocketConfig(RocketConfigError),
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
            MinerErrorKind::Msg(ref message) => write!(f, "{:?}", message),
            MinerErrorKind::Io(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::Ipfs(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::Config(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::SubXt(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::Hex(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::Codec(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::None(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::RocketConfig(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::PublicError(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::SecretString(ref e) => write!(f, "{:?}", e),
            MinerErrorKind::Reqwest(ref e) => write!(f, "{:?}", e),
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

impl From<ReqwestError> for MinerError {
    fn from(e: ReqwestError) -> Self {
        Self { kind: MinerErrorKind::Reqwest(e), source: None }
    }
}

impl From<FromHexError> for MinerError {
    fn from(e: FromHexError) -> Self {
        Self { kind: MinerErrorKind::Hex(e), source: None }
    }
}

impl From<SecretStringError> for MinerError {
    fn from(e: SecretStringError) -> Self {
        Self { kind: MinerErrorKind::SecretString(e), source: None }
    }
}

impl From<PublicError> for MinerError {
    fn from(e: PublicError) -> Self {
        Self { kind: MinerErrorKind::PublicError(e), source: None }
    }
}

impl From<CodecError> for MinerError {
    fn from(e: CodecError) -> Self {
        Self { kind: MinerErrorKind::Codec(e), source: None }
    }
}

impl From<NoneError> for MinerError {
    fn from(e: NoneError) -> Self {
        Self { kind: MinerErrorKind::None(e), source: None }
    }
}

impl From<RocketConfigError> for MinerError {
    fn from(e: RocketConfigError) -> Self {
        Self { kind: MinerErrorKind::RocketConfig(e), source: None }
    }
}


impl From<::std::io::Error> for MinerError {
    fn from(e: ::std::io::Error) -> Self {
        Self { kind: MinerErrorKind::Io(e), source: None }
    }
}

// #[macro_export]
// macro_rules! bail {
//     ($e:expr) => {
//         return Err($e.into());
//     };
//     ($fmt:expr, $($arg:tt)+) => {
//         return Err(format!($fmt, $($arg)+).into());
//     };
// }

/// Prints a "backtrace" of some `Error`.
pub fn log_backtrace(e: &MinerError) {
    log::error!("Error: {}", e);
    // bail!("Error: {}", e);
}

pub type Result<T> = ::std::result::Result<T, MinerError>;