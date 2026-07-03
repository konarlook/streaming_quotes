use clap::builder::Str;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read};
use thiserror::Error;

pub struct Tickers {
    tickers: HashSet<String>,
}

impl Tickers {
    pub fn new(tick: Vec<String>) -> Self {
        Self {
            tickers: HashSet::from_iter(tick),
        }
    }
}

impl Tickers {
    pub fn include(&self, ticker: Vec<String>) -> bool {
        for tick in ticker {
            if !self.tickers.contains(tick.as_str()) {
                return false;
            }
        }
        true
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
