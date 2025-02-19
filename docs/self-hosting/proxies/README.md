# Proxies

Maintaining a good IP reputation is hard. Reacher integrates seamlessly with SOCKS5 proxies.

## What is a SOCKS5 Proxy?

A **SOCKS5 proxy** is a flexible proxy protocol that supports various types of traffic, including SMTP. When using it for email verifications, the reputation of the **proxy’s IP** is what matters, not your own IP. This is crucial for maintaining deliverability and avoiding issues like blacklisting.

Choosing a reputable 3rd-party proxy will greatly improve the quality of your email verification results. For a list of recommended proxies, see [#which-3rd-party-proxies-does-reacher-recommend](./#which-3rd-party-proxies-does-reacher-recommend "mention")

{% hint style="info" %}
SMTP email verifications are not possible via a traditional HTTP proxy.
{% endhint %}

## Setting up a Proxy

Once you've purchased a proxy, run the Docker command (see how in [install.md](../install.md "mention")) and pass the following flags:

* `-e RCH__PROXY__HOST=<host>`: The IP or hostname of the proxy server.
* `-e RCH__PROXY__PORT=<port>`: The corresponding port.
* `-e RCH__PROXY__USERNAME=<username>`: Optional. A username to authenticate the proxy.
* `-e RCH__PROXY__PASSWORD=<password>`: Optional. The corresponding password.
* `-e RCH__HELLO_NAME=<domain>`: The identifier to use during the "HELO/EHLO" step. It should match a domain name owned by the proxy. Ask your proxy provider about this setting.
* `-e RCH__FROM_EMAIL=<email>`: The email to use during the "MAIL FROM" step. It should be an email from the same domain as the HELLO\_NAME. Ask your proxy provider about this setting.

{% hint style="info" %}
If you're using the [commercial-license-trial.md](../licensing/commercial-license-trial.md "mention"), these fields are already populated with the built-in proxy. However, you can overwrite them by passing these flags again, pointing to a proxy of your own choosing.
{% endhint %}

## Which 3rd-party proxies does Reacher recommend?

Reacher has been working closely with [Proxy4Smtp](https://www.proxy4smtp.com) since early 2024. The service is run by Jon, an email verification expert who has had over 10 years experience in the industry. He understands the complications and challenges that arise from large scale SMTP connections. His proxies integrate seamlessly with Reacher, and are tailored for B2B emails.

Ask [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention") for an introduction to Jon.

## Configure Reacher to respect Proxy limits

If using proxies from [Proxy4Smtp](https://www.proxy4smtp.com/), we highly recommend respecting the following concurrency and throttling limits, to avoid getting the IPs that you purchased blacklisted:

* **10,000 email verifications per IP per day**. You may buy as much IPs as you wish, but the total number of email verifications should respect this limit to avoid getting blacklisted.
* **60 email verifications per IP per minute**. Usage spikes are easily detectable by email providers, which will proceed to flag these IPs, so we highly recommend to smoothen out the email verification stream.
* **5 concurrent verifications per IP at any time**. Given that each email verification takes in average a handful of seconds, this setting should work well with the above setting of 60 verifications per minute.

To estimate how many IPs you should buy, we recommend starting from your expected monthly volume, and calculate the number of IPs you need from there. For example, if you're aiming for 10 millions verifications per month, we recommend buying 33 or 34 IPs:

```
10,000,000 emails per month / 30 = 33,000 emails per day / 10000 = 33 IPs
```

The throttling limits of 10000 per day and 60 per minute are already baked in as defaults into Reacher. However, if you're managing scaling yourself ([option-1-manage-scaling-yourself.md](../scaling-for-production/option-1-manage-scaling-yourself.md "mention")), make sure that at any time you have as many Reacher instances as IPs you bought. This will make sure that the concurrency of 5 is respected.
