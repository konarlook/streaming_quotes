use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use streaming_quotes::net::handler::handle_command;
use streaming_quotes::protocol::quote::StockQuote;
use streaming_quotes::sender::QuoteSender;
use streaming_quotes::tickers::ReadTickerError;
use streaming_quotes::{Registry, tickers};

const TICKERS_UPDATE_TIMEOUT: Duration = Duration::from_secs(1);
const CLEAR_TIMEOUT: Duration = Duration::from_secs(5);

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
    #[arg(long, default_value = "127.0.0.1:9000")]
    udp_addr: SocketAddr,
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

    let registry = Arc::new(Registry::new());

    let tcp_listener = TcpListener::bind(args.server_addr)?;
    let sender = Arc::new(QuoteSender::new(args.udp_addr)?);

    println!(
        "Server listening on {} (tcp) and {} (udp)",
        args.server_addr, args.udp_addr
    );

    {
        let tickers = Arc::clone(&a_tickers);
        let registry = Arc::clone(&registry);
        let sender = Arc::clone(&sender);
        thread::spawn(move || {
            loop {
                for ticker in &tickers.tickers {
                    let tick = StockQuote::random(ticker);
                    registry.route(tick, &sender);
                }
                thread::sleep(TICKERS_UPDATE_TIMEOUT);
            }
        });
    }

    {
        let registry = Arc::clone(&registry);
        thread::spawn(move || {
            loop {
                registry.clear_stale();
                thread::sleep(CLEAR_TIMEOUT);
            }
        });
    }

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                let ticks = Arc::clone(&a_tickers);
                let registry = Arc::clone(&registry);

                thread::spawn(move || {
                    if let Err(e) = handle_command(stream, ticks, registry) {
                        eprintln!("Command error: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e)
            }
        }
    }

    Ok(())
}
