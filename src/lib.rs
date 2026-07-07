use crate::protocol::quote::StockQuote;
use crate::sender::QuoteSender;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub mod net;
pub mod protocol;
pub mod receiver;
pub mod sender;
pub mod tickers;

const STATE_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug)]
pub struct Subscription {
    pub ticker: HashSet<String>,
    pub last_ping: Instant,
}

#[derive(Default, Debug)]
pub struct Registry {
    inner: Mutex<HashMap<SocketAddr, Subscription>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, addr: SocketAddr, tickers: Vec<String>) {
        let mut mux = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        mux.insert(
            addr,
            Subscription {
                ticker: HashSet::from_iter(tickers),
                last_ping: Instant::now(),
            },
        );
    }

    pub fn update_ping(&self, addr: SocketAddr) {
        let mut mux = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(sub) = mux.get_mut(&addr) {
            sub.last_ping = Instant::now();
        }
    }

    pub fn unregister(&self, addr: SocketAddr) {
        let mut mux = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        mux.remove(&addr);
    }

    pub fn clear_stale(&self) {
        let mut mux = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let _ = mux.retain(|_, v| v.last_ping.elapsed() < STATE_TIMEOUT);
    }

    pub fn route(&self, quote: StockQuote, sender: &Arc<QuoteSender>) {
        let mux = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        for (addr, sub) in mux.iter() {
            if sub.ticker.contains(&quote.ticker) {
                if let Err(e) = sender.send(quote.clone(), addr) {
                    eprintln!("Send error: {:?}", e)
                }
            }
        }
    }
}
