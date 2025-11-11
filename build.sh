#!/bin/bash

# Discord Interaction Handler Build Script for AWS Lambda
# Copyright (C) 2023-2025  Joe McNally
# Licensed under GPLv3

set -e

echo "Building Discord Interaction Handler for AWS Lambda..."

# Check if cargo-lambda is installed
if ! command -v cargo-lambda &> /dev/null; then
    echo "cargo-lambda not found. Installing..."
    pip3 install cargo-lambda
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf target/lambda

# Build for AWS Lambda (ARM64)
echo "Building Rust Lambda function (ARM64)..."
cargo lambda build --release --arm64

echo "Build complete!"
echo "Lambda deployment package: target/lambda/discord-interaction-handler/bootstrap"
echo ""
echo "To deploy with Terraform:"
echo "  cd terraform"
echo "  terraform init"
echo "  terraform plan"
echo "  terraform apply"
