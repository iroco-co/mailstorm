# mailstorm

This is an mail injector based on hyper and [Stalwart Labs](https://github.com/stalwartlabs) mail libraries.

# How it works?

## For SMTP inbound mail

It reads mail samples from a directory and randomly send them to the configured SMTP url. 

It is multithreaded and will send mails concurrently with `worker_nb` threads.

It sends them with an average of `worker_pace` seconds between each for each worker.

## For IMAP reading

To be done