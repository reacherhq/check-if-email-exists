# Understanding "is\_reachable"

## `is_reachable`?

Reacher provides a confidence score for how likely an email is to be delivered. This score is shown in the `is_reachable` field and can have four values:

* **`safe`**: This email is very likely to be delivered (bounce rate below 2%). Some bounces may still rarely occur due to IP blacklisting.
* **`invalid`**: This email will almost certainly not be delivered.
* **`risky`**: The email exists but may have problems that could cause issues like bounces or low engagement. These could be:
  * A temporary (disposable) email
  * A shared account (e.g., support@ or admin@)
  * A catch-all address (accepts all emails for a domain)
  * A full inbox
* **`unknown`**: Sometimes, the email provider blocks our real-time verification, so we can’t determine deliverability. If you encounter this, let me know by email at amaury@reacher.email. I'm constantly working on ways to fix these issues case-by-case.

## Full Response

The full response contains more details about the email verification. It is provided in the following JSON format, with each field offering relevant information about the email deliverability.

```json
{
  // The input email address that was checked
  "input": "someone@gmail.com",
  
  // The deliverability status of the email (safe, risky, invalid or unknown)
  "is_reachable": "invalid",
  
  "misc": {
    // Indicates if the email is a disposable (temporary) email
    "is_disposable": false,
    
    // Indicates if the email is a role-based account (e.g., admin@, support@)
    "is_role_account": false,
    
    // The URL to the Gravatar associated with this email, if available
    "gravatar_url": null,
    
    // Information from the "Have I Been Pwned" breach database, if applicable
    "haveibeenpwned": null
  },
  
  "mx": {
    // Whether the domain's MX (Mail Exchange) server accepts email
    "accepts_mail": true,
    
    // A list of MX records for the domain (email servers that handle mail)
    "records": [
      "gmail-smtp-in.l.google.com.",
      "alt3.gmail-smtp-in.l.google.com.",
      "alt2.gmail-smtp-in.l.google.com.",
      "alt4.gmail-smtp-in.l.google.com.",
      "alt1.gmail-smtp-in.l.google.com."
    ]
  },
  
  "smtp": {
    // Indicates if the SMTP server can be connected to
    "can_connect_smtp": true,
    
    // Whether the inbox for this email address is full
    "has_full_inbox": false,
    
    // Indicates if the domain uses a catch-all email address (accepts mail for any address)
    "is_catch_all": false,
    
    // Whether the email is deliverable based on SMTP verification
    "is_deliverable": false,
    
    // Whether the email address is disabled or inactive
    "is_disabled": true
  },
  
  "syntax": {
    // The original email address being checked
    "address": "someone@gmail.com",
    
    // The domain part of the email (e.g., gmail.com)
    "domain": "gmail.com",
    
    // Whether the email has valid syntax (e.g., correct format)
    "is_valid_syntax": true,
    
    // The username part of the email (before the @ symbol)
    "username": "someone",
    
    // The normalized version of the email (no extra spaces, proper formatting)
    "normalized_email": "someone@gmail.com",
    
    // A suggested correction if the email syntax was incorrect (null if no suggestion)
    "suggestion": null
  },
    
  "smtp": {
    // Details of the SMTP verification method used
    "verif_method": {
      // The type of verification (SMTP in this case)
      "type": "Smtp",
      
      // The SMTP server that was contacted for verification
      "host": "alt1.gmail-smtp-in.l.google.com.",
      
      // The port used to connect to the SMTP server
      "port": 25,
      
      // Indicates if a proxy was used during the verification
      "used_proxy": false
    }
  },
    
  "debug": {
    // The server used to process the email verification
    "server_name": "backend1-ovh",
    
    // The time the verification process started
    "start_time": "2024-09-18T21:53:16.012753011Z",
    
    // The time the verification process ended
    "end_time": "2024-09-18T21:53:16.350005307Z",
    
    "duration": {
      // The total time taken for the verification in seconds and nanoseconds
      "secs": 0,
      "nanos": 337252296
    },
  }
}

```

You can also check the [openapi.md](../advanced/openapi.md "mention")specification.
