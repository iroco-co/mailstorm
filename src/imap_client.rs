use async_imap::{Client, Session};
use async_native_tls::TlsStream;
use tokio::net::TcpStream;

pub struct ImapClient {
    user: String,
    client: Result<Session<TlsStream<TcpStream>>, (async_imap::error::Error, Client<TlsStream<TcpStream>>)>
}

impl ImapClient {
    pub async fn try_new(imap_host: &str, user: &str, password: &str) -> Self {
        let imap_addr = (imap_host, 993);
        let tcp_stream = TcpStream::connect(imap_addr).await.unwrap();
        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls.connect(imap_host, tcp_stream).await.unwrap();

        let client = Client::new(tls_stream);
        debug!("connected to {}:{}", imap_addr.0, imap_addr.1);

        Self { user: user.to_string(), client: client.login(user, password).await}
    }

    pub async fn run_loop(&mut self) {
        match &mut self.client {
            Ok(ref mut session) => {
                info!("user {} logged in into IMAP server", self.user);
                session.select("INBOX").await.unwrap();
                loop {
                    // do idle...
                }
            },
            Err((err, _)) => {
                error!("user {} login failed {:?}", self.user, err);
            }
        }
    }
}