# Debugging Reacher

## How to check if port 25 is open?

When choosing a server to install Reacher on, you need to make sure that the server itself has port `25` open AND the chosen cloud provider allows outbound port `25` connections on its network.

To test this, there are two methods:

1. `curl` (try this first)
2. `telnet` (more advanced)

#### 1. Test port 25 with `curl`

Paste the following command in the shell of your server.

```bash
curl -sSf --verbose -k smtp://alt1.gmail-smtp-in.l.google.com:25 --ssl-reqd --mail-from test@gmail.com --mail-rcpt test@gmail.com
```

<details>

<summary>✅ Here's the expected output when port 25 works:</summary>

```bash
* About to connect() to alt1.gmail-smtp-in.l.google.com port 25 (#0)
*   Trying 142.250.153.26...
* Connected to alt1.gmail-smtp-in.l.google.com (142.250.153.26) port 25 (#0)
< 220 mx.google.com ESMTP he11-20020a1709073d8b00b006e862100d5bsi2937572ejc.396 - gsmtp
> EHLO reacher
< 250-mx.google.com at your service, [176.31.197.159]
< 250-SIZE 157286400
< 250-8BITMIME
< 250-STARTTLS
< 250-ENHANCEDSTATUSCODES
< 250-PIPELINING
< 250-CHUNKING
< 250 SMTPUTF8
> STARTTLS
< 220 2.0.0 Ready to start TLS
* Initializing NSS with certpath: sql:/etc/pki/nssdb
* skipping SSL peer certificate verification
* SSL connection using TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
* Server certificate:
* 	subject: CN=mx.google.com
* 	start date: May 04 17:12:30 2022 GMT
* 	expire date: Jul 27 17:12:29 2022 GMT
* 	common name: mx.google.com
* 	issuer: CN=GTS CA 1C3,O=Google Trust Services LLC,C=US
> EHLO reacher
< 250-mx.google.com at your service, [176.31.197.159]
< 250-SIZE 157286400
< 250-8BITMIME
< 250-ENHANCEDSTATUSCODES
< 250-PIPELINING
< 250-CHUNKING
< 250 SMTPUTF8
> MAIL FROM:<test@gmail.com>
< 250 2.1.0 OK he11-20020a1709073d8b00b006e862100d5bsi2937572ejc.396 - gsmtp
> RCPT TO:<test@gmail.c>
< 550-5.1.1 The email account that you tried to reach does not exist. Please try
< 550-5.1.1 double-checking the recipient's email address for typos or
< 550-5.1.1 unnecessary spaces. Learn more at
< 550 5.1.1  https://support.google.com/mail/?p=NoSuchUser he11-20020a1709073d8b00b006e862100d5bsi2937572ejc.396 - gsmtp
* RCPT failed: 550
> QUIT
< 221 2.0.0 closing connection he11-20020a1709073d8b00b006e862100d5bsi2937572ejc.396 - gsmtp
* Closing connection 0
curl: (55) RCPT failed: 55
```

</details>

#### 2. Test port 25 with `telnet`

Paste the following command in the shell of your server.

```bash
telnet alt1.gmail-smtp-in.l.google.com 25
```

<details>

<summary>✅ Click to see expected output when port 25 is open:</summary>

```bash
# This means that connection to port 25 on Google's server is established.
Trying 142.250.153.26...
Connected to alt1.gmail-smtp-in.l.google.com.
Escape character is '^]'.
220 mx.google.com ESMTP t2-20020a056402524200b0041d70e3a2b0si10608932edd.55 - gsmtp
# You can type 'QUIT' to quit this prompt
```

</details>

<details>

<summary>❌ Click to see unsuccessful output:</summary>

```bash
Trying 142.250.153.26...
# This step can hang for a couple of seconds...
telnet: Unable to connect to remote host: Connection refused
```

</details>

## **Which cloud providers have port 25 open?**

Here are details about some of the most well-known providers:

* :warning: AWS: Needs an application to open port 25, link to [apply](https://aws.amazon.com/premiumsupport/knowledge-center/ec2-port-25-throttle/) (Dec 2020).
* :warning: Digital Ocean: Your account needs to be 60d old, then you can [apply](https://www.digitalocean.com/community/questions/how-i-can-open-port-25-please?answer=67100) to open port 25 (Mar 2020).
* ❌ GCP: Port 25 closed, [source](https://cloud.google.com/compute/docs/tutorials/sending-mail).
* ❌ Heroku: Starting from July 2021, Heroku blocks port 25 intermittently according to [this document](https://help.heroku.com/IR3S6I5X/problem-in-sending-e-mails-through-smtp).
* :warning: Hetzner: Port 25 open according to [unofficial source](https://www.reddit.com/r/hetzner/comments/lb2o13/does\_hetzner\_block\_port\_25/) (Feb 2021), but seems now that you need to request manually.
* :warning: Linode: Port 25 closed for new accounts, but can be opened if reverse DNS is set up correctly, [source](https://www.linode.com/docs/guides/running-a-mail-server/#sending-email-on-linode=) (Apr 2022).
* :white\_check\_mark: OVH: Port 25 open on new instances, but outbound port 25 traffic is monitored to prevent spam (May 2022).
* :warning: Vultr: Create support ticket to open port 25, [source](https://www.vultr.com/docs/what-ports-are-blocked/), though [recent reports](https://github.com/LukeSmithxyz/emailwiz/issues/172) (May 2022) show that they won’t do it anymore.

## **How can I debug `"is_reachable": "unknown"`?**

In most cases, if you have a `TimeoutError`, it means that port 25 is closed. Refer to the questions above about opening port 25.

In other cases, you can enable the `RUST_LOG=debug` environment variable on your server or on your Docker container. Then, by looking at the logs when performing an email verification, you can debug why `is_reachable` is unknown. Some example of logs you might find:

*   IP Blacklisted:

    `5.7.1 Service unavailable; Client host [<YOUR_IP>] is blacklisted. Visit https://www.sophos.com/en-us/threat-center/ip-lookup.aspx?ip=<YOUR_IP> to request delisting`
* Your `FROM` field does not match the reverse DNE\
  `(mxgmx117) Nemesis ESMTP Service not available; No SMTP service;`

## Contact me

If you encounter any issue that you don't know how to solve, simply send me an email to [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").
