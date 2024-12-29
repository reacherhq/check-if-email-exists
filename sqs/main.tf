provider "aws" {
  region = "eu-west-3"
}

resource "aws_sqs_queue" "example_queue" {
  name = "check-email-queue"
}

resource "aws_iam_role" "lambda_execution_role" {
  name = "lambda_execution_role"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "lambda.amazonaws.com"
      },
      "Effect": "Allow"
    }
  ]
}
EOF
}

resource "aws_iam_policy" "lambda_policy" {
  name        = "lambda_sqs_cloudwatch_policy"
  description = "IAM policy for Lambda SQS and CloudWatch"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "sqs:ReceiveMessage",
        "sqs:DeleteMessage",
        "sqs:GetQueueAttributes"
      ],
      "Resource": "${aws_sqs_queue.example_queue.arn}"
    },
    {
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "attach_lambda_policy" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.lambda_policy.arn
}

resource "aws_lambda_function" "lambda_function" {
  function_name = "sqs-task-check-email"
  role          = aws_iam_role.lambda_execution_role.arn
  handler       = "function.handler"
  runtime       = "provided.al2"

  image_uri = "reacherhq/sqs"

  environment {
    variables = {
      QUEUE_URL = aws_sqs_queue.example_queue.url
    }
  }
}

resource "aws_lambda_event_source_mapping" "sqs_trigger" {
  event_source_arn = aws_sqs_queue.example_queue.arn
  function_name    = aws_lambda_function.lambda_function.arn
}

resource "aws_ecr_repository" "lambda_repo" {
  name = "reacherhq/sqs"
}

output "queue_url" {
  value = aws_sqs_queue.example_queue.url
}
