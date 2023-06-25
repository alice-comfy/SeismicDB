use std::error;
use std::fmt;

#[derive(Debug)]
pub enum SeismicError {
    ServerError(String),
    DBNotFoundError(String),
    ConnectionError,
    SerialError,
    JsonError,
}
use self::SeismicError::*;

impl error::Error for SeismicError {
    fn description(&self) -> &str {
        match *self {
            ServerError(ref msg) => &msg,
            DBNotFoundError(ref dbname) => &dbname,
            ConnectionError => "Error connecting to seismicdb",
            SerialError => "Error serializing/deserializing",
            JsonError => "Error serializing/deserializing json",
        }
    }
}

impl fmt::Display for SeismicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerError(ref msg) => write!(f, "SeismicError: {}", msg),
            DBNotFoundError(ref dbname) => write!(f, "DBNotFoundError: {}", dbname),
            ConnectionError => write!(f, "ConnectionError"),
            SerialError => write!(f, "SerialError"),
            JsonError => write!(f, "JsonError"),
        }
    }
}

impl From<std::io::Error> for SeismicError {
    fn from(_: std::io::Error) -> Self {
        SeismicError::SerialError
    }
}