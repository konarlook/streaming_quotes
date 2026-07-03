use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("quote must be <name>|<price>|<volume>|<dt>, got {0}")]
    EmptyField(String),
    #[error("field {field} is not a valid ({value}): {source}")]
    InvalidField {
        field: &'static str,
        value: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command `{0}` not found")]
    UnknownCommand(String),
    #[error(transparent)]
    InvalidUdpAddr(#[from] std::net::AddrParseError),
    #[error("invalid STREAM args. Use STREAM <udp_addr> <ticker...>")]
    InvalidStreamArgs,
    #[error("empty ticker list")]
    EmptyTicker,
    #[error("ticker {0} not found in database")]
    UnknownTicker(String),
}
