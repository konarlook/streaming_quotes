use std::collections::HashSet;
use std::io::BufRead;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TickerError {
    #[error("unknown ticker {0}")]
    UnknownName(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Tickers {
    pub tickers: HashSet<String>,
}

impl Tickers {
    pub fn new(ticks: Vec<String>) -> Self {
        Self {
            tickers: HashSet::from_iter(ticks),
        }
    }

    pub fn find_unknown(&self, ticker: &Vec<String>) -> Option<Vec<String>> {
        let mut not_included = Vec::new();
        for tick in ticker {
            if !self.tickers.contains(tick) {
                not_included.push(tick.clone());
            }
        }
        if not_included.is_empty() {
            return None;
        }
        Some(not_included)
    }
}

#[derive(Debug, Error)]
pub enum ReadTickerError {
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error("empty tickers file")]
    EmptyFile,
    #[error("file not found")]
    FileNotFound,
}

pub fn read_tickers_from<R: BufRead>(reader: &mut R) -> Result<Vec<String>, ReadTickerError> {
    let mut tickers: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line: Vec<String> = line
            .split(",")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if line.is_empty() {
            continue;
        }
        tickers.extend(line)
    }
    if tickers.is_empty() {
        return Err(ReadTickerError::EmptyFile);
    }
    Ok(tickers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::Cursor;
    use std::io::{BufReader, Read};

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "test"))
        }
    }

    #[rstest]
    #[case("APL,GPN,VLU")]
    #[case("APL, GPN, VLU")]
    #[case(
        r#"
        APL
        GPN
        VLU
    "#
    )]
    fn test_read_tickers(#[case] raw: &str) {
        let data = raw;

        let mut cursor = Cursor::new(data);
        let tickers = read_tickers_from(&mut cursor);
        assert!(tickers.is_ok());
        assert_eq!(tickers.unwrap(), vec!["APL", "GPN", "VLU"])
    }

    #[rstest]
    fn test_empty_tickers() {
        let data = "";

        let mut cursor = Cursor::new(data);
        let tickers = read_tickers_from(&mut cursor);
        assert!(tickers.is_err());
        assert!(matches!(tickers.err(), Some(ReadTickerError::EmptyFile)))
    }

    #[rstest]
    fn test_read_error() {
        let fake_reader = FailingReader;
        let mut buf = BufReader::new(fake_reader);
        let result = read_tickers_from(&mut buf);
        assert!(result.is_err());
        assert!(matches!(result.err(), Some(ReadTickerError::File(_))));
    }
}
