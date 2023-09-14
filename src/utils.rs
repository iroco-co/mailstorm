use mail_parser::HeaderValue;

static AROBASE: char = '\u{40}';
pub(crate) fn get_recipients(recipient_header: &HeaderValue) -> Vec<String> {
    match recipient_header {
        HeaderValue::AddressList(list) => list.into_iter().map(|a| a.address.as_ref().unwrap().to_string()).collect(),
        HeaderValue::Address(addr) => vec![addr.address.as_ref().unwrap().to_string()],
        _ => Vec::new()
    }
}
pub(crate) fn get_domain_name(email: &String) -> Option<&str> {
    email.split_once(AROBASE).and_then(|t| Some(t.1))
}

pub(crate) fn replace<T>(source: &[T], from: &[T], to: &[T]) -> Vec<T> where T: Clone + PartialEq {
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


#[cfg(test)]
mod test {
    use mail_parser::{Addr, Group};
    use super::*;

    #[test]
    fn get_recipients_no_recipients() {
        assert_eq!(get_recipients(
            &HeaderValue::AddressList(vec![])),
                   vec![] as Vec<String>)
    }

    #[test]
    fn get_recipients_one_recipient() {
        assert_eq!(get_recipients(
            &HeaderValue::Address(
                Addr::new(None, "foo@bar.com"),
            )),
                   vec!["foo@bar.com"])
    }

    #[test]
    fn get_recipients_two_recipients() {
        assert_eq!(get_recipients(
            &HeaderValue::AddressList(vec![
                Addr::new("Foo".into(), "foo@bar.com"),
                Addr::new(None, "baz@bar.com"),
            ])),
                   vec!["foo@bar.com", "baz@bar.com"])
    }

    #[ignore]
    #[test]
    fn get_recipients_two_groups() {
        assert_eq!(get_recipients(
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
    fn test_get_domain_name() {
        assert_eq!(get_domain_name(&"foo@bar.com".to_string()).unwrap(), "bar.com".to_string());
        assert_eq!(get_domain_name(&"not_email".to_string()), None);
    }

    #[test]
    fn search_replace_bytes() {
        assert_eq!(replace("Date: Thu,  7 Sep 2023 15:16:52 +0000
        Message-ID: <id>
        X-Mailer: Thunderbird".as_bytes(), "id".as_bytes(), "this_is_a_new_id".as_bytes()),
                   "Date: Thu,  7 Sep 2023 15:16:52 +0000
        Message-ID: <this_is_a_new_id>
        X-Mailer: Thunderbird".as_bytes())
    }
}