/*
 * Discord Interaction Handler Terraform Outputs
 * Copyright (C) 2023-2025  Joe McNally
 * Licensed under GPLv3
 */

output "lambda_function_name" {
  description = "Name of the Discord interaction handler Lambda function"
  value       = aws_lambda_function.discord_handler.function_name
}

output "lambda_function_arn" {
  description = "ARN of the Discord interaction handler Lambda function"
  value       = aws_lambda_function.discord_handler.arn
}

output "api_gateway_url" {
  description = "URL of the API Gateway endpoint (use this as your Discord Interactions Endpoint URL)"
  value       = "${aws_api_gateway_stage.discord_stage.invoke_url}/interactions"
}

output "api_gateway_id" {
  description = "ID of the API Gateway REST API"
  value       = aws_api_gateway_rest_api.discord_api.id
}

output "api_gateway_stage_name" {
  description = "Name of the API Gateway stage"
  value       = aws_api_gateway_stage.discord_stage.stage_name
}

output "lambda_role_arn" {
  description = "ARN of the Lambda execution role"
  value       = aws_iam_role.lambda_role.arn
}

output "cloudwatch_log_group_name" {
  description = "Name of the CloudWatch log group for Lambda function"
  value       = aws_cloudwatch_log_group.lambda_log_group.name
}

output "discord_endpoint_url" {
  description = "Complete Discord Interactions Endpoint URL to configure in Discord Developer Portal"
  value       = "${aws_api_gateway_stage.discord_stage.invoke_url}/interactions"
}
