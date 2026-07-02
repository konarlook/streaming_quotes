use crate::protocol::errors::ParseError;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp_ms: u64,
}

impl StockQuote {
    pub fn to_wire_line(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp_ms
        )
    }

    pub fn from_wire_line(line: &str) -> Result<Self, ParseError> {
        let result = line
            .trim()
            .split('|')
            .map(|s| s.trim())
            .collect::<Vec<&str>>();
        if result.len() != 4 {
            return Err(ParseError::EmptyField(line.to_string()));
        }
        Ok(Self {
            ticker: result[0].to_string(),
            price: Self::parse_field("price", result[1])?,
            volume: Self::parse_field("value", result[2])?,
            timestamp_ms: Self::parse_field("timestamp_ms", result[3])?,
        })
    }

    fn parse_field<T>(field: &'static str, raw: &str) -> Result<T, ParseError>
    where
        T: std::str::FromStr,
        T::Err: Error + Sync + Send + 'static,
    {
        raw.parse::<T>().map_err(|e| ParseError::InvalidField {
            field,
            value: raw.to_string(),
            source: Box::new(e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("test|2.2|1|1", Ok(
        StockQuote{ticker: "test".to_string(), price: 2.2, volume: 1, timestamp_ms:1 }
    ))]
    fn from_wire_line_test_happy_path(
        #[case] raw: &str,
        #[case] expected: Result<StockQuote, ParseError>,
    ) {
        let got = StockQuote::from_wire_line(raw);
        let got = got.unwrap();
        let expected = expected.unwrap();
        assert_eq!(expected, got);
    }

    #[rstest]
    #[case("test|abc|1|1", "price", true)]
    #[case("test|1|2", "", false)]
    fn from_wire_line_test_error(
        #[case] raw: &str,
        #[case] expected: &str,
        #[case] field_err: bool,
    ) {
        let got = StockQuote::from_wire_line(raw);
        if field_err {
            assert!(matches!(
                got.unwrap_err(),
                ParseError::InvalidField { field, .. } if field == expected,
            ));
        } else {
            assert!(matches!(got.unwrap_err(), ParseError::EmptyField(..)))
        }
    }
}
