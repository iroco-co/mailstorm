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
    queue: Sender<Message<'a>>,
    fixed_pace: bool
}

impl <'a> PaceMaker<'a> {
    pub fn new(queue: Sender<Message<'a>>, mail_dir: String, pace_seconds: f32, fixed_pace: bool) -> PaceMaker<'a> {
        Self { queue, mail_dir, pace_seconds, fixed_pace, messages: Vec::new()}
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
        info!("Loaded {} emails", self.messages.len());
        Ok(self.messages.len())
    }

    pub async fn run_loop(&self) {
        info!("pacemaker loop with pace {}s fixed: {}", self.pace_seconds, self.fixed_pace);
        loop {
            let sleep_duration = if self.fixed_pace {
                Duration::from_millis((self.pace_seconds * 1000.0).round() as u64)
            } else {
                let between_0_1: f64 = random::<f64>();
                let random_millis: u64 = (between_0_1 * f64::from(self.pace_seconds) * 2.0 * 1000.0).round() as u64;
                Duration::from_millis(random_millis)
            };
            sleep(sleep_duration).await;
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
