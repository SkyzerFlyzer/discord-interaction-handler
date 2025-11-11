# Discord Interaction Handler

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![AWS Lambda](https://img.shields.io/badge/AWS-Lambda-orange.svg)](https://aws.amazon.com/lambda/)

A high-performance, Rust-based Discord Interaction Handler designed for deployment on AWS Lambda. This project handles Discord interactions with full support for all Discord API interaction types and automatically routes complex interactions to dedicated Lambda functions.

## Features

- ✅ **Full Discord API Support** - Implements all Discord interaction types:
  - Ping (Type 1)
  - Application Commands (Type 2)
  - Message Components (Type 3)
  - Application Command Autocomplete (Type 4)
  - Modal Submit (Type 5)

- 🔒 **Secure** - Ed25519 signature verification for all incoming requests

- 🚀 **High Performance** - Written in Rust for maximum speed and minimal cold start times

- 🔄 **Auto-routing** - Automatically invokes Lambda functions based on:
  - Command names (`discord-command-<name>`)
  - Component custom IDs (`discord-component-<custom_id>`)
  - Modal custom IDs (`discord-modal-<custom_id>`)
  - Autocomplete handlers (`discord-autocomplete-<command>`)

- 📦 **Infrastructure as Code** - Complete Terraform configuration for easy deployment

- 🧪 **Well-tested** - Comprehensive unit and integration tests

- 📝 **Production-ready** - Includes CloudWatch logging, monitoring, and error handling

## Architecture

```
┌─────────┐         ┌──────────────┐         ┌──────────────────────┐         ┌─────────────────┐
│ Discord │────────▶│ API Gateway  │────────▶│  Main Lambda Handler │────────▶│ Command Lambdas │
└─────────┘         └──────────────┘         └──────────────────────┘         └─────────────────┘
                                                       │
                                                       │
                                                       ▼
                                              ┌─────────────────┐
                                              │ CloudWatch Logs │
                                              └─────────────────┘
```

The main handler:
1. Verifies Discord request signatures using Ed25519
2. Handles Ping interactions immediately
3. Routes other interactions to appropriate Lambda functions
4. Returns formatted responses to Discord

## Quick Start

### Prerequisites

- Rust 1.70 or later
- AWS CLI configured with credentials
- Terraform 1.0 or later
- Discord Application (create at [discord.com/developers](https://discord.com/developers/applications))

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/discord-interaction-handler.git
   cd discord-interaction-handler
   ```

2. **Build the Lambda function**
   ```bash
   # Linux/macOS
   ./build.sh

   # Windows
   .\build.ps1
   ```

3. **Configure Terraform**
   ```bash
   cd terraform
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your Discord public key and AWS settings
   ```

4. **Deploy to AWS**
   ```bash
   terraform init
   terraform apply
   ```

5. **Configure Discord**
   - Copy the `discord_endpoint_url` from Terraform output
   - Go to Discord Developer Portal → Your Application → General Information
   - Paste the URL into "Interactions Endpoint URL"
   - Save changes

For detailed deployment instructions, see [DEPLOYMENT.md](DEPLOYMENT.md).

## Lambda Function Naming Convention

The handler automatically routes interactions to Lambda functions based on their type:

| Interaction Type | Lambda Function Name | Example |
|-----------------|---------------------|---------|
| Application Command | `discord-command-<name>` | `discord-command-ping` |
| Message Component | `discord-component-<custom_id>` | `discord-component-approve` |
| Modal Submit | `discord-modal-<custom_id>` | `discord-modal-user_info` |
| Autocomplete | `discord-autocomplete-<name>` | `discord-autocomplete-search` |

### Component Custom IDs with Arguments

You can pass arguments in custom IDs using colons as separators:
- Custom ID: `approve:user123:request456`
- Lambda function invoked: `discord-component-approve`
- The full interaction (including all arguments) is passed to the Lambda

## Project Structure

```
discord-interaction-handler/
├── src/
│   ├── main.rs                 # Lambda entry point
│   ├── handler.rs              # Request handler
│   ├── lambda_invoker.rs       # Lambda invocation logic
│   └── discord/
│       ├── mod.rs              # Discord module
│       ├── types.rs            # Complete Discord API types
│       └── verification.rs     # Ed25519 signature verification
├── tests/
│   └── integration_tests.rs    # Integration tests
├── terraform/
│   ├── main.tf                 # Main Terraform configuration
│   ├── variables.tf            # Input variables
│   ├── outputs.tf              # Output values
│   └── terraform.tfvars.example
├── .github/
│   └── workflows/
│       └── deploy.yml          # CI/CD pipeline
├── Cargo.toml                  # Rust dependencies
├── build.sh                    # Build script (Linux/macOS)
├── build.ps1                   # Build script (Windows)
├── DEPLOYMENT.md               # Detailed deployment guide
└── README.md                   # This file
```

## Development

### Building Locally

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Formatting

```bash
cargo fmt
```

### Building for Lambda

```bash
cargo lambda build --release --arm64
```

## Discord API Type Coverage

This implementation includes complete type definitions for:

- **Interactions**: All 5 interaction types
- **Commands**: Chat input, User, and Message commands
- **Components**: Buttons, Select Menus (all types), Text Inputs, Action Rows
- **Responses**: All 10 interaction callback types
- **Data Structures**: Users, Members, Channels, Messages, Embeds, etc.

## Configuration

### Environment Variables

The Lambda function uses the following environment variables:

- `PUBLIC_KEY` (Required) - Your Discord application's public key
- `RUST_LOG` (Optional) - Log level (error, warn, info, debug, trace)
- `ENVIRONMENT` (Optional) - Environment name (dev, staging, prod)

### Terraform Variables

Configure these in `terraform/terraform.tfvars`:

```hcl
aws_region         = "us-east-1"
environment        = "prod"
project_name       = "discord-interaction-handler"
discord_public_key = "your_public_key_here"
lambda_timeout     = 30
lambda_memory_size = 256
log_retention_days = 7
rust_log_level     = "info"
```

## Monitoring and Debugging

### CloudWatch Logs

View Lambda logs:
```bash
aws logs tail /aws/lambda/discord-interaction-handler-prod --follow
```

### API Gateway Logs

View API Gateway logs:
```bash
aws logs tail /aws/apigateway/discord-interaction-handler-prod --follow
```

### Testing Your Endpoint

After deployment, test the endpoint:
```bash
curl -X POST https://your-api-gateway-url/interactions \
  -H "Content-Type: application/json" \
  -d '{"type":1}'
```

## Example Handler Lambda Function

Here's an example Python Lambda function for handling a command:

```python
# discord-command-ping Lambda function
import json

def lambda_handler(event, context):
    # event contains the full Discord interaction
    interaction = event

    return {
        "type": 4,  # ChannelMessageWithSource
        "data": {
            "content": f"Pong! 🏓 Interaction ID: {interaction['id']}"
        }
    }
```

## CI/CD

The project includes a GitHub Actions workflow that:
1. Runs tests and linting on every push
2. Builds the Lambda function for ARM64
3. Deploys to AWS on push to main/master

To enable:
1. Add secrets to your GitHub repository:
   - `AWS_ACCESS_KEY_ID`
   - `AWS_SECRET_ACCESS_KEY`
   - `AWS_REGION`
   - `DISCORD_PUBLIC_KEY`

## Performance

- **Cold Start**: ~50-100ms (Rust on ARM64)
- **Warm Execution**: ~5-20ms
- **Memory Usage**: Typically 30-50 MB
- **Cost**: Minimal - Most Discord bots stay within AWS free tier

## Security

- ✅ Ed25519 signature verification on all requests
- ✅ Strict request validation
- ✅ Minimal IAM permissions (principle of least privilege)
- ✅ No sensitive data in logs
- ✅ Environment variable encryption at rest

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

Please ensure:
1. All tests pass (`cargo test`)
2. Code is formatted (`cargo fmt`)
3. No clippy warnings (`cargo clippy -- -D warnings`)
4. Documentation is updated

## Upgrading from Go Version

If you're upgrading from the previous Go version:

1. The API is compatible - no changes needed to Discord configuration
2. Lambda function naming is the same
3. The Terraform configuration is more comprehensive
4. Performance improvements: faster cold starts and lower memory usage

## License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)

Copyright (C) 2023-2025  Joe McNally

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

## Acknowledgments

- [Discord API Documentation](https://discord.com/developers/docs/intro)
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek)

## Support

- 📚 [Full Documentation](DEPLOYMENT.md)
- 🐛 [Report Issues](https://github.com/yourusername/discord-interaction-handler/issues)
- 💬 [Discord API Server](https://discord.gg/discord-api)

## Roadmap

- [ ] Add example handler functions in multiple languages
- [ ] Add metrics and dashboards
- [ ] Add rate limiting support
- [ ] Add caching layer for improved performance
- [ ] Add support for Discord's webhook-based follow-up messages

---

**Built with ❤️ and 🦀 Rust**
