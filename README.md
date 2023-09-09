# mailtempest [![CircleCI](https://dl.circleci.com/status-badge/img/gh/iroco-co/mailtempest/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/iroco-co/mailtempest/tree/main)

> If we can’t face it, we can’t escape it  
> But tonight the storm’s come  
> -- Kate Tempest - Tunnel Vision

This is a mail injector based on tokio and [Stalwart Labs](https://github.com/stalwartlabs) mail libraries.

For IMAP it's using [async_imap](https://github.com/async-email/async-imap).

It is intented to test your mail servers and your mail settings.

# How it works?

```shell
mailtempest 0.1.0
Mail injector to generate SMTP/IMAP load to a mail platform

USAGE:
    mailtempest [OPTIONS] <smtp-host> [imap-host]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --mail-dir <mail-dir>            directory where the mails are going to be read. Default to './mails'
        --pace-seconds <pace-seconds>    average pace of injection in second for pace setter. Default to 1s
        --users-csv <users-csv>          CSV file where users login/password can be loaded. Defaults to users.csv
        --worker-nb <worker-nb>          number of workers. Default to nb users

ARGS:
    <smtp-host>    host of the SMTP server
    <imap-host>    host of the IMAP server
```

## For SMTP inbound mail

It reads mail samples from a directory and randomly send them to the configured SMTP url. 

It is multithreaded and will send mails concurrently with `worker_nb` threads.

It sends them with an average of `worker_pace` seconds.

## For IMAP reading

IMAP clients are run if the imap host is provided.

Clients are logging in at launch. Then the INBOX is selected.

The loop is using [IMAP IDLE](https://en.wikipedia.org/wiki/IMAP_IDLE). Each time an email is received it is fetched. 
