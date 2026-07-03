use crate::protocol::errors::CommandError;
use std::net::SocketAddr;

#[derive(Debug, Eq, PartialEq)]
pub enum RequestCommand {
    Stream {
        addr: SocketAddr,
        tickers: Vec<String>,
    },
}

impl std::fmt::Display for RequestCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RequestCommand::Stream {
                addr,
                tickers: ticker,
            } => {
                write!(f, "STREAM {} {}", addr, ticker.join(","))
            }
        }
    }
}

impl std::str::FromStr for RequestCommand {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmd: Vec<&str> = s.split_whitespace().collect();
        match cmd.get(0) {
            Some(&"STREAM") => {
                if cmd.len() != 3 {
                    return Err(CommandError::InvalidStreamArgs);
                }
                let tickers: Vec<String> = cmd[2]
                    .split(",")
                    .map(|r| r.trim().to_string())
                    .filter(|r| !r.is_empty())
                    .collect();
                if tickers.is_empty() {
                    return Err(CommandError::EmptyTicker);
                }
                Ok(RequestCommand::Stream {
                    addr: SocketAddr::from_str(cmd[1])?,
                    tickers,
                })
            }
            Some(command) => Err(CommandError::UnknownCommand(command.to_string())),
            None => Err(CommandError::UnknownCommand(
                "No command specified".to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub enum Response {
    OK,
    ERR(CommandError),
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Response::OK => write!(f, "OK\n"),
            Response::ERR(e) => write!(f, "ERR: {}\n", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::command::{CommandError, RequestCommand};
    use rstest::rstest;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[rstest]
    #[case("STREAM 127.0.0.1:9999 ADDR,TTM", "127.0.0.1:9999", "ADDR,TTM")]
    fn happy_path_test(
        #[case] command: &str,
        #[case] expected_addr: &str,
        #[case] expected_ticker: &str,
    ) {
        let got = RequestCommand::from_str(command);
        assert!(got.is_ok());
        let got = got.unwrap();
        let want = RequestCommand::Stream {
            addr: SocketAddr::from_str(expected_addr).unwrap(),
            tickers: expected_ticker.split(",").map(String::from).collect(),
        };
        assert_eq!(got, want)
    }

    #[test]
    fn unknown_command_test() {
        let got = RequestCommand::from_str("TEST 123.123.0.1:9999 APL");
        assert!(got.is_err());
        assert!(matches!(got.unwrap_err(), CommandError::UnknownCommand(_)));

        let got = RequestCommand::from_str("STREAM APL,TEST");
        assert!(got.is_err());
        assert!(matches!(got.unwrap_err(), CommandError::InvalidStreamArgs));

        let got = RequestCommand::from_str("STREAM 123.0.0 ALP");
        assert!(got.is_err());
        assert!(matches!(got.unwrap_err(), CommandError::InvalidUdpAddr(_)));
    }

    #[rstest]
    #[case("STREAM 127.0.0.1:9999 APL,TTM")]
    fn parse_client_sync(#[case] command: &str) {
        let got = RequestCommand::from_str(command);
        assert_eq!(command, got.unwrap().to_string())
    }
}
