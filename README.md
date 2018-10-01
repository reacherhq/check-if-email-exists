# check-if-email-exists

Check if an email address exists before sending the email. Uses Telnet.

## Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## Usage

Make sure you have Telnet installed.

```bash
./checkEmail.sh you@domain.com
```

Outputs `true` if email exists, `false` if it doesn't.

Some mail providers validate sender's mail address before respond, so you should give a valid mail address as sender's mail.

You can do this by one of the following methods:

```bash
export SENDER=vaild_sender@domain.com
./checkEmail.sh you@domain.com
```

or

```bash
./checkEmail.sh you@domain.com vaild_sender@domain.com
```

## Notes

This is really a draft version of the tool, for my personal uses. If someone wants to improve it (and it's easy), please submit a PR. The code is really easy, but needs some improvement to be deliverable to the public.

The main Bash script (checkEmail.sh) calls an expect script (expectTelnet.sh) to make a Telnet call to the host with ports 587, 465 and 25.

## License

See the LICENSE file.
