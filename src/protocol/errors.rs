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
