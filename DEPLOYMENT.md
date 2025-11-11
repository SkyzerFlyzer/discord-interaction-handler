# Deployment Guide

This guide will walk you through deploying the Discord Interaction Handler to AWS Lambda using Terraform.

## Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **cargo-lambda** - Tool for building Lambda functions
   ```bash
   pip3 install cargo-lambda
   ```

3. **Terraform** (1.0 or later)
   - Download from [terraform.io](https://www.terraform.io/downloads)

4. **AWS CLI** configured with appropriate credentials
   ```bash
   aws configure
   ```

5. **Discord Application** - Create one at [discord.com/developers](https://discord.com/developers/applications)

## Build Process

### Option 1: Using the build script (Recommended)

#### Linux/macOS:
```bash
./build.sh
```

#### Windows:
```powershell
.\build.ps1
```

### Option 2: Manual build

```bash
# Install cargo-lambda if not already installed
pip3 install cargo-lambda

# Build for AWS Lambda (ARM64)
cargo lambda build --release --arm64
```

The compiled Lambda function will be at: `target/lambda/discord-interaction-handler/bootstrap`

## Deployment with Terraform

### 1. Configure Terraform Variables

Copy the example variables file:
```bash
cd terraform
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars` with your values:
```hcl
aws_region         = "us-east-1"
environment        = "prod"
project_name       = "discord-interaction-handler"
discord_public_key = "your_discord_public_key_from_developer_portal"

# Optional: Adjust these as needed
lambda_timeout       = 30
lambda_memory_size   = 256
log_retention_days   = 7
rust_log_level       = "info"
```

### 2. Get Your Discord Public Key

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Select your application
3. Navigate to "General Information"
4. Copy the "Public Key"
5. Add it to your `terraform.tfvars` file

### 3. Initialize Terraform

```bash
cd terraform
terraform init
```

### 4. Review the Deployment Plan

```bash
terraform plan
```

Review the resources that will be created:
- Lambda function (main interaction handler)
- IAM roles and policies
- API Gateway REST API
- CloudWatch log groups

### 5. Deploy to AWS

```bash
terraform apply
```

Type `yes` when prompted to confirm the deployment.

### 6. Get the Endpoint URL

After deployment completes, Terraform will output the Discord endpoint URL:
```
discord_endpoint_url = "https://xxxxxxxxxx.execute-api.us-east-1.amazonaws.com/prod/interactions"
```

Copy this URL.

### 7. Configure Discord

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Select your application
3. Navigate to "General Information"
4. Paste the endpoint URL into the "Interactions Endpoint URL" field
5. Click "Save Changes"

Discord will send a test request to verify your endpoint. If everything is configured correctly, you'll see a green checkmark.

## Creating Lambda Functions for Commands and Components

The interaction handler will invoke Lambda functions based on Discord interactions:

### Command Handlers
- **Naming Convention**: `discord-command-<command_name>`
- **Example**: For a command named "ping", create a Lambda function named `discord-command-ping`

### Component Handlers (Buttons, Select Menus, etc.)
- **Naming Convention**: `discord-component-<custom_id>`
- **Example**: For a button with custom_id "approve", create `discord-component-approve`
- **With Arguments**: Custom IDs can include arguments separated by colons (e.g., "approve:user123:request456")
  - Only the first part is used for the function name: `discord-component-approve`
  - The full custom_id is passed in the payload

### Modal Handlers
- **Naming Convention**: `discord-modal-<custom_id>`
- **Example**: For a modal with custom_id "user_info", create `discord-modal-user_info`

### Autocomplete Handlers
- **Naming Convention**: `discord-autocomplete-<command_name>`
- **Example**: For autocomplete on "search" command, create `discord-autocomplete-search`

### Lambda Function Requirements

Each handler Lambda function should:
1. Accept the full Discord interaction as input (JSON)
2. Return a valid Discord interaction response
3. Have permissions to be invoked by the main handler (automatically granted via Terraform)

Example handler function structure:
```python
import json

def lambda_handler(event, context):
    # event contains the full Discord interaction
    interaction = event

    # Process the interaction
    # ...

    # Return a Discord interaction response
    return {
        "type": 4,  # ChannelMessageWithSource
        "data": {
            "content": "Command processed successfully!"
        }
    }
```

## Monitoring

### CloudWatch Logs

View logs for the main handler:
```bash
aws logs tail /aws/lambda/discord-interaction-handler-prod --follow
```

### API Gateway Logs

View API Gateway logs:
```bash
aws logs tail /aws/apigateway/discord-interaction-handler-prod --follow
```

## Updating the Deployment

After making code changes:

1. Rebuild the Lambda function:
   ```bash
   ./build.sh
   ```

2. Redeploy with Terraform:
   ```bash
   cd terraform
   terraform apply
   ```

## Terraform Outputs

After deployment, you can view all outputs:
```bash
cd terraform
terraform output
```

Available outputs:
- `discord_endpoint_url` - The URL to configure in Discord
- `lambda_function_name` - Name of the Lambda function
- `lambda_function_arn` - ARN of the Lambda function
- `api_gateway_url` - Base URL of the API Gateway
- `cloudwatch_log_group_name` - Name of the log group

## Cleanup

To remove all deployed resources:
```bash
cd terraform
terraform destroy
```

Type `yes` when prompted to confirm.

## Troubleshooting

### Signature Verification Failed
- Verify the `PUBLIC_KEY` environment variable matches your Discord application's public key
- Check that the timestamp header is being passed correctly

### Lambda Function Not Found
- Ensure your command/component handler Lambda functions follow the naming convention
- Verify the functions are deployed in the same AWS region

### Timeout Issues
- Increase `lambda_timeout` in `terraform.tfvars`
- Optimize your handler function code

### Permission Errors
- Verify IAM roles have the correct permissions
- Check CloudWatch logs for detailed error messages

## CI/CD with GitHub Actions

A GitHub Actions workflow is included (`.github/workflows/deploy.yml`) that automatically:
1. Runs tests on every push
2. Builds the Lambda function
3. Deploys to AWS on push to main/master branch

To use it:
1. Add these secrets to your GitHub repository:
   - `AWS_ACCESS_KEY_ID`
   - `AWS_SECRET_ACCESS_KEY`
   - `AWS_REGION`
   - `DISCORD_PUBLIC_KEY`

2. Push to the main branch to trigger deployment

## Architecture Diagram

```
Discord → API Gateway → Main Lambda Handler → Command/Component Lambda Functions
                              ↓
                        CloudWatch Logs
```

The main Lambda handler:
1. Verifies Discord signatures
2. Handles Ping requests immediately
3. Routes other interactions to appropriate Lambda functions
4. Returns responses to Discord
