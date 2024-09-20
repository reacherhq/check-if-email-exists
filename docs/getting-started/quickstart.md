# Verify your 1st email

There are two ways to verify an email with Reacher:

1. Using the Reacher Dashboard (quick & easy).
2. Using the Reacher API (more advanced and powerful).

## 1. Using the Reacher Dashboard

The easiest way to verify an email verification is to create a free account on [https://app.reacher.email](https://app.reacher.email).

Then simply input the email address you want to verify, and click on the "Verify" button:

<figure><img src="../.gitbook/assets/Screenshot 2024-09-18 at 23.36.27.png" alt=""><figcaption><p>A screenshot of the Reacher Dashboard</p></figcaption></figure>

You will get a response with an "`is_reachable`" field, which can take on of the four values: `safe`, `invalid`, `risky` or `unknown`. You can learn more about these 4 values in [is-reachable.md](is-reachable.md "mention").

## 2. Using the Reacher API

{% hint style="info" %}
If you do not know what an API is, you can skip to the next page about [is-reachable.md](is-reachable.md "mention").
{% endhint %}

While the Reacher Dashboard offers a simple way to start verifying emails, the true potential of Reacher lies in its API. Through the API, you can integrate Reacher with your own applications, link it to platforms like Mailchimp or HubSpot, or even sync it with a CRM system.

After creating an account on [https://app.reacher.email](https://app.reacher.email), you will receive an unique API token. You can then run the following `curl` command in your terminal:

```bash
curl -X POST \
    https://api.reacher.email/v0/check_email \
    -H 'content-type: application/json' \
      -H 'authorization: <YOUR_API_TOKEN>' \
      -d '{"to_email": "amaury@reacher.email"}'
```

```json
// Output:
{
    "input":"amaury@reacher.email",
    "is_reachable":"safe",
    // --snip--
}
```
