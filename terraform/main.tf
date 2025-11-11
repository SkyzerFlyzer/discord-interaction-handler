/*
 * Discord Interaction Handler Terraform Configuration
 * Copyright (C) 2023-2025  Joe McNally
 * Licensed under GPLv3
 */

terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.4"
    }
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "discord-interaction-handler"
      Environment = var.environment
      ManagedBy   = "Terraform"
    }
  }
}

# Data source for current AWS account
data "aws_caller_identity" "current" {}

# Data source for AWS region
data "aws_region" "current" {}

# Archive the Lambda deployment package
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = "${path.module}/../target/lambda/bootstrap"
  output_path = "${path.module}/../target/lambda/discord-interaction-handler.zip"
}

# IAM role for the main Lambda function
resource "aws_iam_role" "lambda_role" {
  name = "${var.project_name}-lambda-role-${var.environment}"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

# IAM policy for Lambda to write logs
resource "aws_iam_role_policy_attachment" "lambda_logs" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# IAM policy for Lambda to invoke other Lambda functions
resource "aws_iam_role_policy" "lambda_invoke_policy" {
  name = "${var.project_name}-lambda-invoke-policy-${var.environment}"
  role = aws_iam_role.lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "lambda:InvokeFunction"
        ]
        Resource = [
          "arn:aws:lambda:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:function:discord-command-*",
          "arn:aws:lambda:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:function:discord-component-*",
          "arn:aws:lambda:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:function:discord-modal-*",
          "arn:aws:lambda:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:function:discord-autocomplete-*"
        ]
      }
    ]
  })
}

# CloudWatch Log Group for the main Lambda function
resource "aws_cloudwatch_log_group" "lambda_log_group" {
  name              = "/aws/lambda/${var.project_name}-${var.environment}"
  retention_in_days = var.log_retention_days
}

# Main Discord Interaction Handler Lambda function
resource "aws_lambda_function" "discord_handler" {
  filename         = data.archive_file.lambda_zip.output_path
  function_name    = "${var.project_name}-${var.environment}"
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
  runtime         = "provided.al2023"
  architectures   = ["arm64"]
  timeout         = var.lambda_timeout
  memory_size     = var.lambda_memory_size

  environment {
    variables = {
      PUBLIC_KEY  = var.discord_public_key
      ENVIRONMENT = var.environment
      RUST_LOG    = var.rust_log_level
    }
  }

  depends_on = [
    aws_cloudwatch_log_group.lambda_log_group,
    aws_iam_role_policy_attachment.lambda_logs
  ]
}

# API Gateway REST API
resource "aws_api_gateway_rest_api" "discord_api" {
  name        = "${var.project_name}-api-${var.environment}"
  description = "API Gateway for Discord Interaction Handler"

  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

# API Gateway Resource
resource "aws_api_gateway_resource" "discord_resource" {
  rest_api_id = aws_api_gateway_rest_api.discord_api.id
  parent_id   = aws_api_gateway_rest_api.discord_api.root_resource_id
  path_part   = "interactions"
}

# API Gateway Method (POST)
resource "aws_api_gateway_method" "discord_post" {
  rest_api_id   = aws_api_gateway_rest_api.discord_api.id
  resource_id   = aws_api_gateway_resource.discord_resource.id
  http_method   = "POST"
  authorization = "NONE"
}

# API Gateway Integration with Lambda
resource "aws_api_gateway_integration" "lambda_integration" {
  rest_api_id             = aws_api_gateway_rest_api.discord_api.id
  resource_id             = aws_api_gateway_resource.discord_resource.id
  http_method             = aws_api_gateway_method.discord_post.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.discord_handler.invoke_arn
}

# Lambda permission for API Gateway to invoke
resource "aws_lambda_permission" "api_gateway_invoke" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.discord_handler.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.discord_api.execution_arn}/*/*"
}

# API Gateway Deployment
resource "aws_api_gateway_deployment" "discord_deployment" {
  rest_api_id = aws_api_gateway_rest_api.discord_api.id

  triggers = {
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.discord_resource.id,
      aws_api_gateway_method.discord_post.id,
      aws_api_gateway_integration.lambda_integration.id,
    ]))
  }

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [
    aws_api_gateway_integration.lambda_integration
  ]
}

# API Gateway Stage
resource "aws_api_gateway_stage" "discord_stage" {
  deployment_id = aws_api_gateway_deployment.discord_deployment.id
  rest_api_id   = aws_api_gateway_rest_api.discord_api.id
  stage_name    = var.environment

  xray_tracing_enabled = var.enable_xray_tracing

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.api_gateway_log_group.arn
    format = jsonencode({
      requestId      = "$context.requestId"
      ip             = "$context.identity.sourceIp"
      requestTime    = "$context.requestTime"
      httpMethod     = "$context.httpMethod"
      resourcePath   = "$context.resourcePath"
      status         = "$context.status"
      protocol       = "$context.protocol"
      responseLength = "$context.responseLength"
    })
  }
}

# CloudWatch Log Group for API Gateway
resource "aws_cloudwatch_log_group" "api_gateway_log_group" {
  name              = "/aws/apigateway/${var.project_name}-${var.environment}"
  retention_in_days = var.log_retention_days
}

# Method settings for API Gateway
resource "aws_api_gateway_method_settings" "discord_method_settings" {
  rest_api_id = aws_api_gateway_rest_api.discord_api.id
  stage_name  = aws_api_gateway_stage.discord_stage.stage_name
  method_path = "*/*"

  settings {
    metrics_enabled = true
    logging_level   = var.api_gateway_log_level
  }
}
