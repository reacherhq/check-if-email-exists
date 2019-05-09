# Serverless

This package deploys the `check_if_email_exists` function as a lambda function on AWS via [serverless](https://serverless.com/).

To try it out, follow the steps below:

#### Set up `serverless`

Follow serverless's guide: https://serverless.com/framework/docs/providers/aws/guide/quick-start/.

#### Invoke the function locally

Change `put_your_email_here@gmail.com` to the email your wish to test inside `payload.json`, and run from the root folder:

Note: you need to have Docker installed.

```bash
serverless invoke local -f check_if_email_exists_serverless -d "$(cat serverless/payload.json)"
```

#### Deploy the function to AWS

```bash
serverless deploy
```

#### Invoke the deployed function

```bash
serverless invoke -f check_if_email_exists_serverless -d "$(cat serverless/payload.json)"
```
