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

    let _ = writer.write_all(b"Quote stream started\n")?;
    let _ = writer.flush()?;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                return Ok(());
            }
            Ok(_) => {
                let input = line.trim();
                let result = process_line(input, tick.clone());
                let _ = writer.write_all(result.to_string().as_bytes())?;
                let _ = writer.flush()?;
            }
            Err(e) => return Err(e),
        }
    }
}

fn process_line(raw: &str, ticker: Arc<Tickers>) -> Response {
    match RequestCommand::from_str(raw) {
        Ok(request) => match request {
            RequestCommand::Stream { tickers, .. } => {
                if ticker.include(tickers) {
                    return Response::OK;
                }
                Response::ERR(CommandError::UnknownTicker("".to_string()))
            }
        },
        Err(e) => Response::ERR(e),
    }
}
