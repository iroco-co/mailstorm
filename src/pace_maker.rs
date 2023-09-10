use std::fs;
use std::time::Duration;
use rand::seq::SliceRandom;
use mail_parser::Message;
use rand::random;
use tokio::time::sleep;
use async_channel::Sender;

pub struct PaceMaker<'a> {
    mail_dir: String,
    pace_seconds: f32,
    messages: Vec<Message<'a>>,
    queue: Sender<Message<'a>>
}

impl <'a> PaceMaker<'a> {
    pub fn new(queue: Sender<Message<'a>>, mail_dir: String, pace_seconds: f32) -> PaceMaker<'a> {
        Self { queue, mail_dir, pace_seconds, messages: Vec::new()}
    }

    pub fn load_messages(&mut self) -> Result<usize, std::io::Error> {
        info!("Loading emails from {:?}", self.mail_dir);
        let paths = fs::read_dir(&self.mail_dir)?;
        for path in paths {
            let path = path.unwrap().path();
            let contents = fs::read(path)?;
            let message = Message::parse(contents.as_slice()).unwrap();
            self.messages.push(message.into_owned());
        }
        Ok(self.messages.len())
    }

    pub async fn run_loop(&self) {
        info!("PaceSetter loop with pace {:?}s", self.pace_seconds);
        loop {
            let between_0_1: f64 = random::<f64>();
            let wait_time_millis: u64 = (between_0_1 * f64::from(self.pace_seconds) * 2.0 * 1000.0).round() as u64;
            sleep(Duration::from_millis(wait_time_millis)).await;
            let msg = {
                let mut rng = rand::thread_rng(); // rng should fall out of scope before async
                self.messages.choose(&mut rng)
            };
            match msg {
                Some(message) => self.queue.send(message.to_owned()).await.unwrap(),
                None => error!("cannot pick a message from cache")
            };
        }
    }
}
