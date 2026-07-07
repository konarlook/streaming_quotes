use crate::Registry;
use crate::protocol::command::{RequestCommand, Response};
use crate::protocol::errors::CommandError;
use crate::tickers::Tickers;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::Arc;

pub fn handle_command(
    stream: TcpStream,
    tick: Arc<Tickers>,
    reg: Arc<Registry>,
) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return Ok(()),
            Ok(_) => {
                let input = line.trim();
                println!("DEBUG: command {:?}", input);
                let result = process_line(input, &tick);
                writer.write_all(result.to_string().as_bytes())?;
                writer.flush()?;

                match result {
                    Response::OK { addr, ticks } => {
                        reg.register(addr, ticks);
                        println!("Command successfully registered: {:?}", reg.inner);
                    }
                    Response::PONG(addr) => reg.update_ping(addr),
                    Response::ERR(e) => eprintln!("ERR: {}", e),
                }
            }
            Err(e) => return Err(e),
        }
    }
}

fn process_line(raw: &str, ticker: &Tickers) -> Response {
    match RequestCommand::from_str(raw) {
        Ok(request) => match request {
            RequestCommand::Stream { tickers, addr } => match ticker.find_unknown(&tickers) {
                Some(ticks) => Response::ERR(CommandError::UnknownTicker(ticks.join(","))),
                None => Response::OK {
                    addr,
                    ticks: tickers,
                },
            },
            RequestCommand::Ping(addr) => Response::PONG(addr),
        },
        Err(e) => Response::ERR(e),
    }
}

pub fn handle_client(stream: &mut TcpStream, command: RequestCommand) -> std::io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    writer.write_all(command.to_string().as_bytes())?;
    writer.flush()?;

    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => return Ok(()),
        Ok(_) => {
            let input = line.trim();
            println!("{:?}", line);
            if input.is_empty() {
                writer.flush()?;
            }
            if input.eq_ignore_ascii_case("OK") {
                println!("Client successfully registered!");
                return Ok(());
            }
            if input.eq_ignore_ascii_case("PONG") {
                println!("Client connected!");
                return Ok(());
            }
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
