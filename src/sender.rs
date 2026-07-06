use crate::protocol::quote::StockQuote;
use crate::tickers::Tickers;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

#[derive(Debug)]
pub struct QuoteSender {
    pub socket: UdpSocket,
}

impl QuoteSender {
    pub fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self { socket })
    }

    pub fn send(
        &self,
        quote: StockQuote,
        target: &SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encode = quote.to_wire_line().into_bytes();
        self.socket.send_to(&encode, target)?;
        Ok(())
    }

    pub fn broadcast(
        &self,
        target: &SocketAddr,
        interval: u64,
        tickers: &Tickers,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            for t in tickers.tickers.iter() {
                self.send(StockQuote::random(t), target)?;
            }
            std::thread::sleep(Duration::from_secs(interval))
        }
    }
}
