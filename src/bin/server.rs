use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use streaming_quotes::net::handler::handle_command;
use streaming_quotes::tickers;
use streaming_quotes::tickers::ReadTickerError;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

#[derive(Parser, Debug)]
#[command(about = "Quote streaming server")]
struct Args {
    #[arg(long, short = 'f')]
    ticker_file: PathBuf,
    #[arg(long, default_value = "127.0.0.1:7878")]
    server_addr: SocketAddr,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ticks: tickers::Tickers;
    if args.ticker_file.is_file() {
        let reader = File::open(args.ticker_file)?;
        let mut buf = BufReader::new(reader);
        ticks = tickers::Tickers::new(tickers::read_tickers_from(&mut buf)?);
    } else {
        return Err(ReadTickerError::FileNotFound)?;
    }
    let a_tickers = Arc::new(ticks);

    let listener = TcpListener::bind(args.server_addr)?;
    println!("Server listening on {}", args.server_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let ticks = Arc::clone(&a_tickers);
                thread::spawn(move || {
                    handle_command(stream, ticks).unwrap_err();
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e)
            }
        }
    }

    Ok(())
}
