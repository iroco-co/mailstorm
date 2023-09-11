#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{BufReader, Error, Write};

use async_channel::{Receiver, Sender, unbounded};
use mail_parser::Message;
use structopt::StructOpt;

use crate::mail_reader::MailReader;
use crate::mail_sender::MailSender;
use crate::pace_maker::PaceMaker;

mod pace_maker;
mod mail_sender;
mod mail_reader;

/// Mail injector to generate SMTP/IMAP load to a mail platform.
#[derive(StructOpt, Debug)]
#[structopt(name = "mailtempest")]
struct Args {
    /// host of the SMTP server.
    smtp_host: String,
    /// host of the IMAP server.
    imap_host: Option<String>,
    #[structopt(long)]
    /// directory where the mails are going to be read. Default to './mails'
    mail_dir: Option<String>,
    #[structopt(long)]
    /// CSV file where users login/password can be loaded. Defaults to users.csv
    users_csv: Option<String>,
    #[structopt(long)]
    /// average pace of injection in second for pace maker (float). Default to 1s.
    pace_seconds: Option<f32>,
    #[structopt(long)]
    /// number of workers. Default to nb users.
    worker_nb: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MailtempestConfig {
    pub smtp_host: String,
    pub imap_host: String,
    pub mail_dir: String,
    pub users_csv: String,
    pub worker_nb: u8,
    pub pace_seconds: f32
}

#[derive(Debug, Clone)]
struct MailAccount {
    user: String,
    password: String
}

impl MailAccount {
    fn new(user: &str, password: &str) -> Self {
        Self { user: user.to_string(), password: password.to_string()}
    }
}

impl Args {
    fn to_config(self) -> MailtempestConfig {
        MailtempestConfig {
            smtp_host: self.smtp_host,
            imap_host: match self.imap_host {
                Some(imap_host) => imap_host,
                None => String::new()
            },
            mail_dir: match self.mail_dir {
                Some(mail_dir) => mail_dir,
                None => "./mails".to_string()
            },
            users_csv: match self.users_csv {
                Some(users_csv) => users_csv,
                None => "./users.csv".to_string()
            },
            worker_nb: match self.worker_nb {
                Some(worker_nb) => worker_nb, 
                None => 1 
            },
            pace_seconds: match self.pace_seconds {
                Some(pace_seconds) => pace_seconds,
                None => 1.0
            },
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    init_logs();
    let opt = Args::from_args();
    let config = opt.to_config();
    let mail_accounts = load_users(&config.users_csv).unwrap();
    info!("Running mailtempest with SMTP host={:?} and {:?} worker(s)", config.smtp_host, mail_accounts.len());

    let (sx, rx): (Sender<Message>, Receiver<Message>) = unbounded();

    let mut pace_maker = PaceMaker::new(sx.clone(), config.mail_dir, config.pace_seconds);
    info!("Loaded {} emails", pace_maker.load_messages().unwrap());

    for mail_account in &mail_accounts {
        let mut mail_sender = MailSender::new(rx.clone(),
                                              config.smtp_host.clone(),
                                              mail_account.user.clone(), mail_account.password.clone()).await;
        tokio::task::spawn(async move {
            mail_sender.run_loop().await
        });
    }

    if !config.imap_host.is_empty() {
        for mail_account in mail_accounts {
            let mut mail_reader = MailReader::new(config.imap_host.as_str());
            tokio::task::spawn(async move {
                mail_reader.run_loop(&mail_account.user, &mail_account.password).await
            });
        }
    }
    pace_maker.run_loop().await;
}

fn load_users(file_path: &str) -> Result<Vec<MailAccount>, Error> {
    info!("Loading user accounts from {:?}", file_path);
    let reader = BufReader::new(File::open(file_path)?);
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(reader);
    let mut results: Vec<MailAccount> = vec![];
    for record_res in rdr.records() {
        let record = record_res?;
        results.push(MailAccount::new(record.get(0).unwrap(), record.get(1).unwrap()))
    }
    Ok(results)
}

fn init_logs() {
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: [{:?}] {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    std::thread::current().id(),
                    record.args()
                )
            })
            .init(),
        _ => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} {:<5} {}: [{:?}] {}",
                    buf.timestamp_micros(),
                    buf.default_level_style(record.level())
                        .value(record.level()),
                    record.target(),
                    std::thread::current().id(),
                    record.args()
                )
            })
            .init(),
    };
}
