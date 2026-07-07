use crate::protocol::quote::StockQuote;
use std::net::{SocketAddr, UdpSocket};
use std::thread;

pub struct QuoteReceiver {
    socket: UdpSocket,
}

impl QuoteReceiver {
    pub fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self { socket })
    }

    pub fn recv_loop(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0u8; 1024];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, _)) => {
                    let s = std::str::from_utf8(&buf[..size])?;
                    match StockQuote::from_wire_line(s) {
                        Ok(quote) => {
                            println!("{}", quote.to_wire_line());
                        }
                        Err(e) => {
                            println!("Parse error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving from socket: {}", e);
                }
            }
        }
    }

    pub fn start(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            if let Err(e) = self.recv_loop() {
                eprintln!("Error receiving from socket: {}", e);
            }
        })
    }
}
