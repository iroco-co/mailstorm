use mail_parser::Message as ParsedMessage;
use mail_send::SmtpClientBuilder;
use mail_send::smtp::message::Message;
use async_channel::Receiver;

pub struct MailSender<'a> {
    user: String,
    password: String,
    smtp_host: String,
    queue: Receiver<ParsedMessage<'a>>
}

impl <'a> MailSender<'a> {
    pub async fn new(queue: Receiver<ParsedMessage<'a>>, smtp_host: String, user: String, password: String) -> MailSender<'a> {
        Self { user, password, smtp_host, queue }
    }

    pub async fn run_loop(&mut self) -> () {
        info!("MailSender loop for {:?}", self.user);
        loop {
            match self.queue.recv().await {
                Ok(parsed_message) => {
                    debug!("received {:?} from {:?}", parsed_message.message_id(), parsed_message.from());
                    let message = Message::empty()
                        .from(self.user.clone())
                        .to(self.user.clone())
                        .body(parsed_message.raw_message);
                    SmtpClientBuilder::new(self.smtp_host.clone(), 465)
                        .implicit_tls(true)
                        .credentials((self.user.clone(), self.password.clone()))
                        .connect()
                        .await
                        .unwrap()
                        .send(message)
                        .await
                        .unwrap();
                },
                Err(e) => error!("received error from channel {:?}", e)
            };
        }
    }
}
