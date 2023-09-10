use std::time::Duration;

use async_imap::Client;
use async_imap::extensions::idle::IdleResponse::NewData;
use tokio::{net::TcpStream, task, time::sleep};
use futures::TryStreamExt;

static IMAP_PORT: u16 = 993;

pub struct MailReader {
    imap_host: String
}

impl MailReader {
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
        loop {
            let mut idle = session.idle();
            idle.init().await.unwrap();
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
                    let uids = Self::get_exists_from_idle(&s);
                    session = idle.done().await.unwrap();
                    let messages_stream = session.fetch(uids.join(","), "RFC822").await.unwrap();
                    let messages: Vec<_> = messages_stream.try_collect().await.unwrap();
                    debug!("IDLE read {} messages", messages.len());
                }
                reason => {
                    debug!("IDLE failed {:?}", reason);
                    session = idle.done().await.unwrap();
                }
            }
        }
    }
    fn get_exists_from_idle(idle_data: &str) -> Vec<&str> {
        let lines = idle_data.lines();
        lines.filter(|l| l.contains("EXISTS")).map(|l| {
            l.trim().split(' ').filter(|s| !s.is_empty()).nth(1).unwrap()
        }).collect()
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_exists_from_idle_with_2_lines() {
        assert_eq!(MailReader::get_exists_from_idle("   * 18 EXISTS
        * 1 RECENT
        "), vec!["18"]);
    }
}