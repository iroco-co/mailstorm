mod pace_setter;
mod mail_sender;
mod imap_client;

#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::Write;
use async_channel::{Sender, Receiver, unbounded};
use mail_parser::Message;
use crate::imap_client::ImapClient;
use crate::mail_sender::MailSender;
use crate::pace_setter::PaceSetter;

static MAIL_USERS: [(&str, &str); 2]  = [("user1@domain.com", "mdp1"), ("user2@domain.com", "mdp2")];

/// Mail injector to generate SMTP/IMAP load to a mail platform.
#[derive(StructOpt, Debug)]
#[structopt(name = "mailstorm")]
struct Args {
    /// host of the SMTP server.
    smtp_host: String,
    /// host of the IMAP server.
    imap_host: Option<String>,
    #[structopt(long)]
    /// directory where the mails are going to be read. Default to './mails'
    mail_dir: Option<String>,
    #[structopt(long)]
    /// average pace of injection in second for pace setter. Default to 1s.
    pace_seconds: Option<u8>,
    #[structopt(long)]
    /// number of workers. Default to nb users.
    worker_nb: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MailstormConfig {
    pub smtp_host: String,
    pub imap_host: String,
    pub mail_dir: String,
    pub worker_nb: u8,
    pub pace_seconds: u8
}

impl Args {
    fn to_config(self) -> MailstormConfig {
        MailstormConfig {
            smtp_host: self.smtp_host,
            imap_host: match self.imap_host {
                Some(imap_host) => imap_host,
                None => String::new()
            },
            mail_dir: match self.mail_dir {
                Some(mail_dir) => mail_dir,
                None => "./mails".to_string()
            },
            worker_nb: match self.worker_nb {
                Some(worker_nb) => worker_nb, 
                None => 1 
            },
            pace_seconds: match self.pace_seconds {
                Some(pace_seconds) => pace_seconds,
                None => 1
            },
        }
    }
}

#[tokio::main]
async fn main() {
    init_logs();
    let opt = Args::from_args();
    let config = opt.to_config();
    info!("Running mailstorm with SMTP host={:?} and {:?} worker(s)", config.smtp_host, MAIL_USERS.len());
    let (sx, rx): (Sender<Message>, Receiver<Message>) = unbounded();

    let mut pace_setter = PaceSetter::new(sx.clone(), config.mail_dir, config.pace_seconds);

    for user in MAIL_USERS {
        let mut mail_sender = MailSender::new(rx.clone(),
                                              config.smtp_host.clone(),
                                              user.0.to_string(), user.1.to_string()).await;
        tokio::task::spawn(async move {
            mail_sender.run_loop().await
        });
    }

    if !config.imap_host.is_empty() {
        for user in MAIL_USERS {
            let mut imap_client = ImapClient::new(config.imap_host.as_str());
            tokio::task::spawn(async move {
                imap_client.run_loop(user.0, user.1).await
            });
        }
    }
    pace_setter.load_messages();
    pace_setter.run_loop().await;
}

fn init_logs() {
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            })
            .init(),
        _ => env_logger::init(),
    };
}
