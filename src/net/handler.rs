use crate::protocol::command::{RequestCommand, Response};
use crate::protocol::errors::CommandError;
use crate::tickers::Tickers;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::Arc;

pub fn handle_command(stream: TcpStream, tick: Arc<Tickers>) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => Ok(()),
        Ok(_) => {
            let input = line.trim();
            let result = process_line(input, &tick);
            writer.write_all(result.to_string().as_bytes())?;
            writer.flush()?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn process_line(raw: &str, ticker: &Tickers) -> Response {
    match RequestCommand::from_str(raw) {
        Ok(request) => match request {
            RequestCommand::Stream { tickers, .. } => match ticker.find_unknown(tickers) {
                Some(ticks) => Response::ERR(CommandError::UnknownTicker(ticks.join(","))),
                None => Response::OK,
            },
        },
        Err(e) => Response::ERR(e),
    }
}
