# check-if-email-exists
Check if an email address exists without sending any email. Uses telnet.

## Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a fee. Here is an open-source alternative to those tools.

## Usage

```bash
chmod a+x checkEmail.sh
./checkEmail.sh you@domain.com
```

Outputs true if email exists, false if it doesn't.

## Notes

This is really a draft version of the tool, for my personal uses. If someone wants to improve it (and it's easy), please submit a PR.

The main Bash script (checkEmail.sh) calls a expect script (expectTelnet.sh) to make a Telnet call to the host with ports 587, 465 and 25.

## License

See the LICENSE file.