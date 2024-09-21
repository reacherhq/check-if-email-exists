# Proxies

## What is a SOCKS5 Proxy?

A **SOCKS5 proxy** is a flexible proxy protocol that supports various types of traffic, including SMTP. When using it for email verifications, the reputation of the **proxy’s IP** is what matters, not your own IP. The proxy handles requests on your behalf, which helps protect your actual IP while ensuring verifications go through successfully. This is crucial for maintaining deliverability and avoiding issues like blacklisting.

{% hint style="warning" %}
SMTP email verifications are not possible via a traditional HTTP proxy.
{% endhint %}

## Verify an Email Using a Proxy

Reacher's API allows making an email verification using a proxy. Send a request with the following fields:

```json
{
    "to_email": "someone@gmail.com",
    "proxy": {                        // (optional) SOCK5 proxy to run the verification through, default is empty
        "host": "my-proxy.io",
        "port": 1080,
        "username": "username",       // (optional) Proxy username
        "password": "password"        // (optional) Proxy password
    }
}
```

## Which 3rd-party proxies does Reacher recommend?

Get in touch with [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention") if you are interested in using a proxy.
