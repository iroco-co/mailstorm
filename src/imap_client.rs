use std::time::Duration;

use async_imap::{Client};
use async_imap::extensions::idle::IdleResponse::NewData;
use tokio::{net::TcpStream, task, time::sleep};
use futures::TryStreamExt;
use crate::errors::EmailResult;

static IMAP_PORT: u16 = 993;

pub struct ImapClient {
    session: async_imap::Session<async_native_tls::TlsStream<TcpStream>>
}

impl ImapClient {
    pub async fn try_new(imap_host: &str, user: &str, password: &str) -> EmailResult<Self> {
        let imap_addr = (imap_host, IMAP_PORT);
        let tcp_stream = TcpStream::connect(imap_addr).await?;
        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls.connect(imap_host, tcp_stream).await?;

        let client = Client::new(tls_stream);
        debug!("connected to {}:{}", imap_host, IMAP_PORT);

        let session = client.login(user, password).await.map_err(|(e, _)| e)?;
        info!("user {} logged in into IMAP server", user);

        Ok(Self { session })
    }

    pub async fn wait_for_new_messages(&mut self) {
        self.session.select("INBOX").await.unwrap();
        let mut idle = self.session.idle();
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
                let mut s = idle.done().await.unwrap();
                let messages_stream = s.fetch(uids.join(" "), "RFC822").await.unwrap();
                let messages: Vec<_> = messages_stream.try_collect().await.unwrap();
                debug!("IDLE read {} messages", messages.len());
            }
            reason => {
                info!("IDLE stopped {:?}", reason);
            }
        }
    }

    fn get_exists_from_idle(idle_data: &str) -> Vec<&str> {
        idle_data.lines().filter(|l| l.contains("EXISTS")).map(|l| {
            l.trim().split(' ').filter(|s| !s.is_empty()).nth(1).unwrap()
        }).collect()
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_exists_from_idle_with_2_lines() {
        assert_eq!(ImapClient::get_exists_from_idle("   * 18 EXISTS
        * 1 RECENT
        "), vec!["18"]);
    }
}