use crate::utils;
use async_channel::Receiver;
use mail_parser::{Message as ParsedMessage};
use mail_send::smtp::message::Message;
use mail_send::SmtpClientBuilder;
use uuid::Uuid;



pub struct MailSender<'a> {
    user: String,
    password: String,
    smtp_host: String,
    queue: Receiver<ParsedMessage<'a>>,
}

impl<'a> MailSender<'a> {
    pub async fn new(queue: Receiver<ParsedMessage<'a>>, smtp_host: String, user: String, password: String) -> MailSender<'a> {
        Self { user, password, smtp_host, queue }
    }

    pub async fn run_loop(&mut self) -> () {
        info!("MailSender loop for {:?}", self.user);
        loop {
            match self.queue.recv().await {
                Ok(parsed_message) => {
                    debug!("received {:?} from {:?}", parsed_message.message_id(), parsed_message.from());
                    let mut to_list: Vec<String> = utils::get_recipients(&parsed_message.to());
                    let mut cc_list: Vec<String> = utils::get_recipients(&parsed_message.cc());
                    let mut bcc_list: Vec<String> = utils::get_recipients(&parsed_message.bcc());
                    to_list.append(&mut cc_list);
                    to_list.append(&mut bcc_list);
                    let to_list_same_domain: Vec<&String> = to_list.iter().filter(|email| {
                        email.ends_with(utils::get_domain_name(&self.user).unwrap())
                    }).collect();
                    if to_list_same_domain.is_empty() {
                        warn!("mail id {:?} recipient list is empty for domain {} not sending mail",
                            parsed_message.message_id(), utils::get_domain_name(&self.user).unwrap());
                    } else {
                        let raw_body = utils::replace(parsed_message.raw_message(),
                                                       parsed_message.message_id().unwrap().as_bytes(),
                                                       Uuid::new_v4().to_string().as_bytes());
                        let message = Message::new(self.user.clone(), to_list, raw_body);
                        SmtpClientBuilder::new(self.smtp_host.clone(), 465)
                            .implicit_tls(true)
                            .credentials((self.user.clone(), self.password.clone()))
                            .connect()
                            .await
                            .unwrap()
                            .send(message)
                            .await
                            .unwrap_or_else(| e | error!("error while sending mail {:?}", e));
                    }
                }
                Err(e) => error!("received error from channel {:?}", e)
            };
        }
    }
}
