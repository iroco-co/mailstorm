#[macro_use]
extern crate log;

use std::borrow::Cow;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, Write};
use std::process::exit;

use async_channel::{Receiver, Sender, unbounded};
use mail_parser::Message;
use structopt::StructOpt;
use tokio::runtime;

use crate::mail_reader::MailReader;
use crate::mail_sender::MailSender;
use crate::pace_maker::PaceMaker;

mod pace_maker;
mod mail_sender;
mod mail_reader;
mod utils;

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
    /// there is no random delay between messages. The delay is always pace_seconds.
    fixed_pace: Option<bool>,
    #[structopt(long)]
    /// number of workers.
    workers: Option<usize>,
    #[structopt(long)]
    /// utility prepare command (boolean). It will use the CSV file to replace all the email addresses in the files located in mail directory
    /// and rewrite them with .mt extension
    prepare: Option<bool>
}

#[derive(Debug, Clone)]
pub struct MailtempestConfig {
    pub smtp_host: String,
    pub imap_host: String,
    pub mail_dir: String,
    pub users_csv: String,
    pub workers: usize,
    pub pace_seconds: f32,
    pub fixed_pace: bool,
    pub prepare: bool
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
            workers: match self.workers {
                Some(workers) => workers,
                None => 1 
            },
            pace_seconds: match self.pace_seconds {
                Some(pace_seconds) => pace_seconds,
                None => 1.0
            },
            prepare: match self.prepare {
                Some(prepare) => prepare,
                None => false
            },
            fixed_pace: match self.fixed_pace {
                Some(fixed_pace) => fixed_pace,
                None => false
            },
        }
    }
}

#[tokio::main]
async fn main() {
    init_logs();
    let opt = Args::from_args();
    let config = opt.to_config();
    let mail_accounts = load_users(&config.users_csv).unwrap();
    if config.prepare {
        prepare(mail_accounts, config.mail_dir);
        exit(0);
    }

    info!("Running mailtempest with SMTP host={}, {} users and {} worker(s)", config.smtp_host, mail_accounts.len(), config.workers);
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(config.workers)
        .enable_io()
        .enable_time()
        .build().unwrap();

    let (sx, rx): (Sender<Message>, Receiver<Message>) = unbounded();
    let mut pace_maker = PaceMaker::new(sx.clone(), config.mail_dir, config.pace_seconds, config.fixed_pace);
    pace_maker.load_messages().expect("cannot load messages");

    for mail_account in &mail_accounts {
        let mut mail_sender = MailSender::new(rx.clone(),
                                              config.smtp_host.clone(),
                                              mail_account.user.clone(), mail_account.password.clone()).await;
        rt.spawn(async move {
            mail_sender.run_loop().await
        });
    }

    if !config.imap_host.is_empty() {
        for mail_account in mail_accounts {
            let mut mail_reader = MailReader::new(config.imap_host.as_str());
            rt.spawn(async move {
                mail_reader.run_loop(&mail_account.user, &mail_account.password).await
            });
        }
    }
    pace_maker.run_loop().await;
}

fn prepare(accounts: Vec<MailAccount>, mail_dir: String) {
    let mut iter_mail = accounts.iter().cycle();

    let paths = fs::read_dir(mail_dir).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let contents = fs::read(&path).unwrap();
        let parsed_message = Message::parse(contents.as_slice()).unwrap();
        let mut to_list: Vec<String> = utils::get_recipients(&parsed_message.to());
        let mut cc_list: Vec<String> = utils::get_recipients(&parsed_message.cc());
        let mut bcc_list: Vec<String> = utils::get_recipients(&parsed_message.bcc());
        to_list.append(&mut cc_list);
        to_list.append(&mut bcc_list);
        let mut new_contents = parsed_message.raw_message;
        for email in to_list {
            let cloned_content = new_contents.clone();
            new_contents = Cow::from(utils::replace::<u8>(&cloned_content, email.as_bytes(), iter_mail.next().unwrap().user.as_bytes()));
        }
        let mut new_file_path = path.clone();
        new_file_path.set_extension(".mt");
        fs::write(new_file_path, new_contents).unwrap();
    }
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
