# Multiple Proxies

{% hint style="info" %}
This feature is only available starting from Reacher v0.11.0.
{% endhint %}

For advanced use cases, Reacher supports a routing mechanism to route email verification requests to different proxies depending on the MX host.

The configuration is divided in 2 steps:

1. Define a list of proxies. e.g. `"proxy1"`, `"proxy2"`etc., each with their configuration data.
2. Define routing rules, e.g. `route Gmail to "proxy1"`, `route Hotmail B2B to "proxy2"`etc...

### 1. Define a list of Proxies

The simple Reacher configuration allows to define a default proxy via the following environmental variables:

* `RCH__PROXY__HOST`: The hostname of the proxy.
* `RCH__PROXY__PORT`: The port of the proxy.
* `RCH__PROXY__USERNAME`: (Optional) A username for authentication.
* `RCH__PROXY__PASSWORD`: (Optional) A password for authentication.

On top of the default proxy, Reacher allows you to configure multiple other proxies:

* `RCH__OVERRIDES__PROXIES__{your-proxy-id-uppercase}__HOST`&#x20;
* `RCH__OVERRIDES__PROXIES__{your-proxy-id-uppercase}__PORT`&#x20;
* `RCH__OVERRIDES__PROXIES__{your-proxy-id-uppercase}__USERNAME`&#x20;
* `RCH__OVERRIDES__PROXIES__{your-proxy-id-uppercase}__PASSWORD`&#x20;

Replace `{your-proxy-id-uppercase}`with any unique name of your choosing. We recommend to use `PROXY1`, `PROXY2` for simplicity.

### 2. Define routing rules

Reacher allows to configure custom routing rules based on the MX host. To do so, set the following environment variables:

* `RCH__OVERRIDES__{email-provider}__TYPE=smtp`
* `RCH__OVERRIDES__{email-provider}__PROXY={your-proxy-id-lowercase}`

where you replace:

* `{email-provider}` with one of the following `GMAIL`, `HOTMAILB2B`, `HOTMAILB2C`, `PROOFPOINT`, `MIMECAST`, `YAHOO`.
* `{your-proxy-id-lowercase}` with one of the proxies you defined in step 1. Make sure to respect the lowercase here.

You can define multiple of these `RCH__OVERRIDES__`  environment variables for different MX hosts. All the remaining emails which don't match any of the overrides will go through the default proxy.

### Example

Below is a [Docker Compose](https://docs.docker.com/compose/) file showcasing:

* routing Gmail and Proofpoint emails to proxy1
* routing Hotmail B2B emails to proxy2
* routing everything else to the default proxy

```yaml
services:
  worker:
    image: reacherhq/commercial-license-trial:v0.11.0
    container_name: test
    ports:
      - "8080:8080"
    environment:
      RCH__BACKEND_NAME: backend4-do
      RUST_LOG: reacher=info
      # Default proxy
      RCH__PROXY__HOST: my.default.proxy.com
      RCH__PROXY__PORT: 1081
      RCH__PROXY__USERNAME: user0
      RCH__PROXY__PASSWORD: pass0
      RCH__HELLO_NAME: my.default.proxy.com
      RCH__FROM_EMAIL: hello@my.default.proxy.com
      # Proxy 1
      RCH__OVERRIDES__PROXIES__PROXY1__HOST: 11.22.33.44
      RCH__OVERRIDES__PROXIES__PROXY1__PORT: 1081
      RCH__OVERRIDES__PROXIES__PROXY1__USERNAME: user1
      RCH__OVERRIDES__PROXIES__PROXY1__PASSWORD: pass1
      # Proxy 2
      RCH__OVERRIDES__PROXIES__PROXY2__HOST: 55.66.77.88
      RCH__OVERRIDES__PROXIES__PROXY2__PORT: 1081
      RCH__OVERRIDES__PROXIES__PROXY2__USERNAME: user2
      RCH__OVERRIDES__PROXIES__PROXY2__PASSWORD: pass2
      # Route Google to Proxy 1
      RCH__OVERRIDES__GMAIL__TYPE: smtp
      RCH__OVERRIDES__GMAIL__PROXY: proxy1
      # Route Proofpoint to Proxy 1
      RCH__OVERRIDES__PROOFPOINT__TYPE: smtp
      RCH__OVERRIDES__PROOFPOINT__PROXY: proxy1
      # Route Hotmail B2B to Proxy 2
      RCH__OVERRIDES__HOTMAILB2B__TYPE: smtp
      RCH__OVERRIDES__HOTMAILB2B__PROXY: proxy2
      RCH__OVERRIDES__HOTMAILB2B__HELLO_NAME: my.proxy2.com       # Optionally override the HELO/EHLO name
      RCH__OVERRIDES__HOTMAILB2B__FROM_EMAIL: hello@my.proxy2.com # Optionally override the MAIL FROM email.
      # Worker config
      RCH__WORKER__ENABLE: false
    restart: always
```

