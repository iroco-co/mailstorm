use async_channel::Receiver;
use mail_parser::{HeaderValue, Message as ParsedMessage};
use mail_send::smtp::message::Message;
use mail_send::SmtpClientBuilder;
use uuid::Uuid;

static AROBASE: char = '\u{40}';

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
                    let mut to_list: Vec<String> = Self::get_recipients(&parsed_message.to());
                    let mut cc_list: Vec<String> = Self::get_recipients(&parsed_message.cc());
                    let mut bcc_list: Vec<String> = Self::get_recipients(&parsed_message.bcc());
                    to_list.append(&mut cc_list);
                    to_list.append(&mut bcc_list);
                    let to_list_same_domain: Vec<&String> = to_list.iter().filter(|email| {
                        email.ends_with(Self::get_domain_name(&self.user).unwrap())
                    }).collect();
                    if to_list_same_domain.is_empty() {
                        warn!("mail id {:?} recipient list is empty for domain {} not sending mail",
                            parsed_message.message_id(), Self::get_domain_name(&self.user).unwrap());
                    } else {
                        let raw_body = MailSender::replace(parsed_message.raw_message(),
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
                            .unwrap();
                    }
                }
                Err(e) => error!("received error from channel {:?}", e)
            };
        }
    }

    fn get_recipients(recipient_header: &HeaderValue) -> Vec<String> {
        match recipient_header {
            HeaderValue::AddressList(list) => list.into_iter().map(|a| a.address.as_ref().unwrap().to_string()).collect(),
            HeaderValue::Address(addr) => vec![addr.address.as_ref().unwrap().to_string()],
            _ => Vec::new()
        }
    }
    fn get_domain_name(email: &String) -> Option<&str> {
        email.split_once(AROBASE).and_then(|t| Some(t.1))
    }

    fn replace<T>(source: &[T], from: &[T], to: &[T]) -> Vec<T> where T: Clone + PartialEq {
        let mut result = source.to_vec();
        let from_len = from.len();
        let to_len = to.len();

        let mut i = 0;
        while i + from_len <= result.len() {
            if result[i..].starts_with(from) {
                result.splice(i..i + from_len, to.iter().cloned());
                i += to_len;
            } else {
                i += 1;
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use mail_parser::{Addr, Group};
    use super::*;

    #[test]
    fn get_recipients_no_recipients() {
        assert_eq!(MailSender::get_recipients(
            &HeaderValue::AddressList(vec![])),
                   vec![] as Vec<String>)
    }

    #[test]
    fn get_recipients_one_recipient() {
        assert_eq!(MailSender::get_recipients(
            &HeaderValue::Address(
                Addr::new(None, "foo@bar.com"),
            )),
           vec!["foo@bar.com"])
    }

   #[test]
    fn get_recipients_two_recipients() {
        assert_eq!(MailSender::get_recipients(
            &HeaderValue::AddressList(vec![
                Addr::new("Foo".into(), "foo@bar.com"),
                Addr::new(None, "baz@bar.com"),
            ])),
           vec!["foo@bar.com", "baz@bar.com"])
    }

    #[ignore]
    #[test]
    fn get_recipients_two_groups() {
        assert_eq!(MailSender::get_recipients(
            &HeaderValue::GroupList(
                vec![
                    Group::new("A", vec![Addr::new(None, "bar@foo.com")]),
                    Group::new("B", vec![
                        Addr::new(None, "baz@foo.com"),
                        Addr::new("Qux".into(), "qux@foo.com"),
                    ])
                ])),
               vec!["bar@foo.com", "baz@foo.com", "qux@foo.com"]
            )
    }
    #[test]
    fn get_domain_name() {
        assert_eq!(MailSender::get_domain_name(&"foo@bar.com".to_string()).unwrap(), "bar.com".to_string());
        assert_eq!(MailSender::get_domain_name(&"not_email".to_string()), None);
    }

    #[test]
    fn search_replace_bytes() {
        assert_eq!(MailSender::replace("Date: Thu,  7 Sep 2023 15:16:52 +0000
        Message-ID: <id>
        X-Mailer: Thunderbird".as_bytes(), "id".as_bytes(), "this_is_a_new_id".as_bytes()),
        "Date: Thu,  7 Sep 2023 15:16:52 +0000
        Message-ID: <this_is_a_new_id>
        X-Mailer: Thunderbird".as_bytes())
    }

}