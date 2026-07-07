use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use streaming_quotes::net::handler::handle_client;
use streaming_quotes::protocol::command::RequestCommand;
use streaming_quotes::receiver::QuoteReceiver;
use streaming_quotes::tickers;
use streaming_quotes::tickers::ReadTickerError;

#[derive(Parser, Debug)]
#[command(about = "Quote streaming client")]
struct Args {
    #[arg(long)]
    udp_addr: SocketAddr,
    #[arg(long, default_value = "127.0.0.1:7878")]
    tcp_addr: SocketAddr,
    #[arg(long, short = 'f')]
    ticker_file: PathBuf,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let ticks: Vec<String>;
    if args.ticker_file.is_file() {
        let reader = File::open(args.ticker_file)?;
        let mut buf = BufReader::new(reader);
        ticks = tickers::read_tickers_from(&mut buf)?;
    } else {
        return Err(ReadTickerError::FileNotFound)?;
    }

    let mut tcp_listener = TcpStream::connect(args.tcp_addr)?;
    {
        thread::spawn(move || {
            let command = RequestCommand::Stream {
                addr: args.udp_addr,
                tickers: ticks,
            };
            if let Err(e) = handle_client(&mut tcp_listener, command) {
                eprintln!("STREAM command failed: {}", e);
            }
            loop {
                let command = RequestCommand::Ping(args.udp_addr);
                if let Err(e) = handle_client(&mut tcp_listener, command) {
                    eprintln!("ping failed: {}", e);
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    let receiver = QuoteReceiver::new(args.udp_addr)?;
    let receive_handler = receiver.start();

    if let Err(e) = receive_handler.join() {
        Err(e).unwrap()
    }

    thread::sleep(Duration::from_millis(100));

    Ok(())
}
