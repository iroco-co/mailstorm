use std::time::Duration;

use async_imap::Client;
use async_imap::extensions::idle::IdleResponse::NewData;
use tokio::{net::TcpStream, task, time::sleep};

static IMAP_PORT: u16 = 993;

pub struct ImapClient {
    imap_host: String
}

impl ImapClient {
    pub fn new(imap_host: &str) -> Self {
        Self { imap_host: imap_host.to_string() }
    }

    pub async fn run_loop(&mut self, user: &str, password: &str) {
        let imap_addr = (self.imap_host.clone(), IMAP_PORT);
        let tcp_stream = TcpStream::connect(imap_addr).await.unwrap();
        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls.connect(&self.imap_host, tcp_stream).await.unwrap();

        let client = Client::new(tls_stream);
        debug!("connected to {}:{}", self.imap_host, IMAP_PORT);

        let mut session = client.login(user, password).await.unwrap();
        info!("user {} logged in into IMAP server", user);

        session.select("INBOX").await.unwrap();
        let mut idle = session.idle();
        idle.init().await.unwrap();
        loop {
            let (idle_wait, interrupt) = idle.wait();
            task::spawn(async move {
                debug!("IDLE: waiting for 30s");
                sleep(Duration::from_secs(30)).await;
                debug!("IDLE: waited 30 secs, now interrupting idle");
                drop(interrupt);
            });
            match idle_wait.await.unwrap() {
                NewData(data) => {
                    let s = String::from_utf8(data.borrow_owner().to_vec()).unwrap();
                    debug!("IDLE data:\n{}", s);
                }
                reason => {info!("IDLE failed {:?}", reason)}
            }
        }
    }
}