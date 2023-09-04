# mailstorm

This is a mail injector based on hyper and [Stalwart Labs](https://github.com/stalwartlabs) mail libraries.

For IMAP it's using [async_imap](https://github.com/async-email/async-imap).

# How it works?

## For SMTP inbound mail

It reads mail samples from a directory and randomly send them to the configured SMTP url. 

It is multithreaded and will send mails concurrently with `worker_nb` threads.

It sends them with an average of `worker_pace` seconds between each for each worker.

## For IMAP reading

IMAP clients are run if the imap host is provided.

When it is, clients are logging at launch. Then the INBOX is selected.

The loop is using IMAP IDLE. Each time an email is received it is fetched. 
