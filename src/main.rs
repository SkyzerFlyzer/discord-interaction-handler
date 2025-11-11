/*
 * Discord Interaction Handler is a Rust project intended for deployment on AWS Lambda to handle Discord Interactions.
 *     Copyright (C) 2023-2025  Joe McNally
 *
 *     This program is free software: you can redistribute it and/or modify
 *     it under the terms of the GNU General Public License as published by
 *     the Free Software Foundation, either version 3 of the License, or
 *     (at your option) any later version.
 *
 *     This program is distributed in the hope that it will be useful,
 *     but WITHOUT ANY WARRANTY; without even the implied warranty of
 *     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *     GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

mod discord;
mod handler;
mod lambda_invoker;

use aws_config::BehaviorVersion;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_http::{run, service_fn, Error};
use tracing::info;

use crate::handler::handle_request;
use crate::lambda_invoker::LambdaInvoker;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    info!("Discord Interaction Handler starting...");

    // Load the public key from environment variables
    let public_key = std::env::var("PUBLIC_KEY").expect("PUBLIC_KEY environment variable not set");

    // Initialize AWS SDK
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let lambda_client = LambdaClient::new(&config);
    let lambda_invoker = LambdaInvoker::new(lambda_client);

    info!("AWS Lambda client initialized");

    // Run the Lambda function
    run(service_fn(|event| async {
        handle_request(event, &public_key, &lambda_invoker).await
    }))
    .await
}
