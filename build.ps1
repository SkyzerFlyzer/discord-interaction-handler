# Discord Interaction Handler Build Script for AWS Lambda (Windows)
# Copyright (C) 2023-2025  Joe McNally
# Licensed under GPLv3

Write-Host "Building Discord Interaction Handler for AWS Lambda..." -ForegroundColor Green

# Check if cargo-lambda is installed
if (!(Get-Command cargo-lambda -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-lambda not found. Installing..." -ForegroundColor Yellow
    pip3 install cargo-lambda
}

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Green
if (Test-Path target/lambda) {
    Remove-Item -Recurse -Force target/lambda
}

# Build for AWS Lambda (ARM64)
Write-Host "Building Rust Lambda function (ARM64)..." -ForegroundColor Green
cargo lambda build --release --arm64

Write-Host "Build complete!" -ForegroundColor Green
Write-Host "Lambda deployment package: target/lambda/discord-interaction-handler/bootstrap"
Write-Host ""
Write-Host "To deploy with Terraform:" -ForegroundColor Cyan
Write-Host "  cd terraform"
Write-Host "  terraform init"
Write-Host "  terraform plan"
Write-Host "  terraform apply"
