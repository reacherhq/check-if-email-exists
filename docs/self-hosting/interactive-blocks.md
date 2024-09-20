# IP Health Maintenance

When running Reacher at scale, managing your **IP health** is crucial to maintaining high email verification accuracy and avoiding potential issues such as blacklisting. Since Reacher connects to email servers to verify addresses, the health of the IPs making these connections directly affects the success rate of your verification requests.

Here are some best practices for maintaining IP health.

## **Rotate IP Addresses**

Constantly making requests from the same IP can lead to throttling or blacklisting, especially if you're performing high-volume verifications. Using an IP rotation service or proxy pools can help distribute the load and reduce the risk of any single IP being blocked.

## Monitor IP Reputation

Regularly check the reputation of the IP addresses you're using. Tools like Sender Score can help monitor your IP reputation. If one or more of your IPs gets flagged, it can impact the accuracy of email verifications, as some servers may block connections from poor IPs.

## **Use Dedicated IPs**

Consider using dedicated IPs for email verification. Shared IPs can sometimes have degraded reputations due to the behavior of other users sharing the same IP range. Dedicated IPs ensure that you maintain control over your reputation.

## Warm Up IPs

If you’re starting with a fresh set of IPs, begin with a lower verification volume and gradually increase the load. This process, known as **IP warming**, helps avoid immediate blacklisting and allows you to build a solid reputation with email servers over time.

## Throttle Verification Requests

Sending too many requests from a single IP in a short period can trigger rate limits or spam filters on email servers. Implement rate-limiting to space out requests and ensure smoother interactions with email servers.

## Too Much Work? Use Proxies!

Deploying Reacher generally requires a one-time setup with minimal ongoing maintenance. However, managing IP addresses can be more time-consuming over the long run. If you prefer to avoid the complexities of IP management, you can route all SMTP requests through a third-party service that handles IP health and reputation management for you. Read more about [proxies.md](proxies.md "mention").
