provider "aws" {
  region = var.aws_region
}

# Define variables for reuse
variable "aws_region" {
  default = "eu-west-3"
}

variable "queue_name" {
  default = "check-email-queue"
}

variable "lambda_name" {
  default = "lambda-task-check-email"
}

variable "repository_name" {
  default = "reacherhq/sqs"
}

# Create an SQS Queue
resource "aws_sqs_queue" "check_email_queue" {
  name = var.queue_name

  tags = {
    Environment = "Production"
    ManagedBy   = "Terraform"
  }
}

# IAM Role for Lambda execution
resource "aws_iam_role" "lambda_execution_role" {
  name = "lambda_execution_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Action = "sts:AssumeRole",
        Principal = {
          Service = "lambda.amazonaws.com"
        },
        Effect = "Allow"
      }
    ]
  })

  tags = {
    Environment = "Production"
    ManagedBy   = "Terraform"
  }
}

# IAM Policy for Lambda
resource "aws_iam_policy" "lambda_policy" {
  name        = "lambda_sqs_cloudwatch_policy"
  description = "IAM policy for Lambda SQS and CloudWatch"

  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = [
          "sqs:ReceiveMessage",
          "sqs:DeleteMessage",
          "sqs:GetQueueAttributes"
        ],
        Resource = aws_sqs_queue.check_email_queue.arn
      },
      {
        Effect = "Allow",
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ],
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })

  tags = {
    Environment = "Production"
    ManagedBy   = "Terraform"
  }
}

# Attach IAM policy to Lambda role
resource "aws_iam_role_policy_attachment" "attach_lambda_policy" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.lambda_policy.arn
}

# Lambda Function
resource "aws_lambda_function" "lambda_task_check_email" {
  function_name = var.lambda_name
  role          = aws_iam_role.lambda_execution_role.arn
  handler       = "function.handler"
  runtime       = "provided.al2"

  # ECR repository image
  package_type = "Image"
  image_uri = "${aws_ecr_repository.lambda_ecr_repo.repository_url}:latest"

  environment {
    variables = {
      QUEUE_URL = aws_sqs_queue.check_email_queue.url
    }
  }

  timeout = 120 # Timeout set to 2 minutes, which corresponds to the max time one email verification should run, plus buffer.

  tags = {
    Environment = "Production"
    ManagedBy   = "Terraform"
  }
}

# Connect SQS Queue to Lambda as event source
resource "aws_lambda_event_source_mapping" "sqs_trigger" {
  event_source_arn = aws_sqs_queue.check_email_queue.arn
  function_name    = aws_lambda_function.lambda_task_check_email.arn

  batch_size = 1
}

# ECR Repository for Lambda image
resource "aws_ecr_repository" "lambda_ecr_repo" {
  name = var.repository_name

  tags = {
    Environment = "Production"
    ManagedBy   = "Terraform"
  }
}

# Output SQS Queue URL
output "queue_url" {
  value       = aws_sqs_queue.check_email_queue.url
  description = "The URL of the SQS queue."
}
