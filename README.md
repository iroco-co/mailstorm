# mailtempest [![CircleCI](https://dl.circleci.com/status-badge/img/gh/iroco-co/mailtempest/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/iroco-co/mailtempest/tree/main)

> If we can’t face it, we can’t escape it  
> But tonight the storm’s come  
> -- Kate Tempest - Tunnel Vision

This is a mail injector based on tokio and [Stalwart Labs](https://github.com/stalwartlabs) mail libraries.

For IMAP it's using [async_imap](https://github.com/async-email/async-imap).

It is intented to test your mail servers and your mail settings.

# How it works?

## For SMTP inbound mail

It reads mail samples from a directory and randomly send them to the configured SMTP url. 

It is multithreaded and will send mails concurrently with `worker_nb` threads.

It sends them with an average of `worker_pace` seconds.

## For IMAP reading

IMAP clients are run if the imap host is provided.

Clients are logging in at launch. Then the INBOX is selected.

The loop is using [IMAP IDLE](https://en.wikipedia.org/wiki/IMAP_IDLE). Each time an email is received it is fetched. 
