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

use lambda_http::{Body, Error, Request, Response};
use serde_json::json;
use tracing::{error, info, warn};

use crate::discord::{verify_discord_signature, Interaction, InteractionResponse, InteractionType};
use crate::lambda_invoker::LambdaInvoker;

/// Handles incoming Discord interaction requests
///
/// This function:
/// 1. Verifies the Discord signature
/// 2. Parses the interaction
/// 3. Handles Ping interactions immediately
/// 4. Delegates other interactions to Lambda functions
///
/// # Arguments
/// * `event` - The Lambda HTTP request
/// * `public_key` - The Discord application public key
/// * `lambda_invoker` - The Lambda invoker for handling interactions
///
/// # Returns
/// * `Response<Body>` - The HTTP response to return to Discord
pub async fn handle_request(
    event: Request,
    public_key: &str,
    lambda_invoker: &LambdaInvoker,
) -> Result<Response<Body>, Error> {
    info!("Received Discord interaction request");

    // Extract headers
    let headers = event.headers();
    let signature = headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let timestamp = headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Get the raw body
    let body_bytes = match event.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes,
        Body::Empty => b"",
    };
    let body_str = String::from_utf8_lossy(body_bytes);

    // Verify the signature
    if signature.is_empty() {
        warn!("Missing signature header");
        return Ok(build_error_response(
            401,
            "Missing signature header".to_string(),
        ));
    }

    if timestamp.is_empty() {
        warn!("Missing timestamp header");
        return Ok(build_error_response(
            401,
            "Missing timestamp header".to_string(),
        ));
    }

    if let Err(e) = verify_discord_signature(public_key, signature, timestamp, &body_str) {
        error!("Signature verification failed: {}", e);
        return Ok(build_error_response(
            401,
            "Invalid request signature".to_string(),
        ));
    }

    info!("Signature verified successfully");

    // Parse the interaction
    let interaction: Interaction = match serde_json::from_str(&body_str) {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to parse interaction: {}", e);
            return Ok(build_error_response(400, "Invalid request body".to_string()));
        }
    };

    info!(
        "Parsed interaction type: {:?}, ID: {}",
        interaction.interaction_type, interaction.id
    );

    // Handle the interaction based on type
    match interaction.interaction_type {
        InteractionType::Ping => {
            info!("Handling Ping interaction");
            let response = InteractionResponse::pong();
            Ok(build_json_response(200, response))
        }
        InteractionType::ApplicationCommand
        | InteractionType::MessageComponent
        | InteractionType::ModalSubmit
        | InteractionType::ApplicationCommandAutocomplete => {
            // Invoke the appropriate Lambda function
            match lambda_invoker.invoke_for_interaction(&interaction).await {
                Ok(lambda_response) => {
                    info!("Lambda invocation successful");
                    Ok(build_json_response(200, lambda_response))
                }
                Err(e) => {
                    error!("Lambda invocation failed: {}", e);
                    // Return a user-friendly error message to Discord
                    let response = InteractionResponse::ephemeral_message(
                        "An error occurred while processing your interaction. Please try again later.".to_string(),
                    );
                    Ok(build_json_response(200, response))
                }
            }
        }
    }
}

/// Builds a JSON response
fn build_json_response<T: serde::Serialize>(status: u16, body: T) -> Response<Body> {
    let body_json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::Text(body_json))
        .unwrap()
}

/// Builds an error response
fn build_error_response(status: u16, message: String) -> Response<Body> {
    let body = json!({
        "error": message
    });

    build_json_response(status, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_error_response() {
        let response = build_error_response(400, "Test error".to_string());
        assert_eq!(response.status(), 400);
    }

    #[test]
    fn test_build_json_response() {
        let data = json!({"test": "value"});
        let response = build_json_response(200, data);
        assert_eq!(response.status(), 200);
    }
}
