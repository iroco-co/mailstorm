#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::Write;

/// Mail injector to generate SMTP/IMAP load to a mail platform.
#[derive(StructOpt, Debug)]
#[structopt(name = "mailstorm")]
struct Args {
    /// smtp uri
    smtp_uri: String,
    /// imap_uri 
    imap_uri: Option<String>,
    #[structopt(long)]
    /// average pace of injection in second for each worker. Default to 1s.
    worker_pace: Option<u8>,
    #[structopt(long)]
    /// number of workers. Default to 1.
    worker_nb: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MailstormConfig {
    pub smtp_uri: String,
    pub imap_uri: String,
    pub worker_nb: u8,
    pub worker_pace: u8
}

impl Args {
    fn to_config(self) -> MailstormConfig {
        MailstormConfig {
            smtp_uri: self.smtp_uri,
            imap_uri: match self.imap_uri {
                Some(imap_uri) => imap_uri,
                None => String::new()
            },
            worker_nb: match self.worker_nb {
                Some(worker_nb) => worker_nb, 
                None => 1 
            },
            worker_pace: match self.worker_pace {
                Some(worker_pace) => worker_pace,
                None => 1
            },
        }
    }
}

fn main() {
    init_logs();
    let opt = Args::from_args();

    let config = opt.to_config();
    info!("Running mailstorm with SMTP url={:?} and {:?} worker(s)", config.smtp_uri, config.worker_nb);
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
