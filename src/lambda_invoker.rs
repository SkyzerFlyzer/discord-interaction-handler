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

use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::InvocationType;
use aws_sdk_lambda::Client as LambdaClient;
use serde_json::Value;
use thiserror::Error;
use tracing::{error, info};

use crate::discord::{Interaction, InteractionType};

#[derive(Error, Debug)]
pub enum LambdaInvokerError {
    #[error("AWS SDK error: {0}")]
    AwsSdkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("No command name found in interaction")]
    NoCommandName,
    #[error("No custom_id found in component interaction")]
    NoCustomId,
}

/// Invokes an AWS Lambda function for a Discord interaction
pub struct LambdaInvoker {
    client: LambdaClient,
}

impl LambdaInvoker {
    /// Creates a new LambdaInvoker with the provided Lambda client
    pub fn new(client: LambdaClient) -> Self {
        Self { client }
    }

    /// Invokes the appropriate Lambda function based on the interaction type
    ///
    /// For ApplicationCommand interactions: invokes a Lambda named after the command
    /// For MessageComponent interactions: invokes a Lambda based on the custom_id
    /// For ModalSubmit interactions: invokes a Lambda based on the custom_id
    /// For ApplicationCommandAutocomplete: invokes a Lambda named after the command with "_autocomplete" suffix
    ///
    /// # Arguments
    /// * `interaction` - The Discord interaction to process
    ///
    /// # Returns
    /// * `Ok(Value)` - The response from the Lambda function
    /// * `Err(LambdaInvokerError)` - If invocation fails
    pub async fn invoke_for_interaction(
        &self,
        interaction: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        match interaction.interaction_type {
            InteractionType::ApplicationCommand => {
                self.invoke_command_lambda(interaction).await
            }
            InteractionType::MessageComponent => {
                self.invoke_component_lambda(interaction).await
            }
            InteractionType::ModalSubmit => self.invoke_modal_lambda(interaction).await,
            InteractionType::ApplicationCommandAutocomplete => {
                self.invoke_autocomplete_lambda(interaction).await
            }
            InteractionType::Ping => {
                // Ping interactions are handled directly, no Lambda invocation needed
                Ok(serde_json::json!({"type": 1}))
            }
        }
    }

    /// Invokes a Lambda function for an application command
    async fn invoke_command_lambda(
        &self,
        interaction: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        let command_name = interaction
            .data
            .as_ref()
            .and_then(|d| d.name.as_ref())
            .ok_or(LambdaInvokerError::NoCommandName)?;

        let function_name = format!("discord-command-{}", command_name);
        info!("Invoking Lambda function: {}", function_name);

        self.invoke_lambda(&function_name, interaction).await
    }

    /// Invokes a Lambda function for a component interaction
    async fn invoke_component_lambda(
        &self,
        interaction: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        let custom_id = interaction
            .data
            .as_ref()
            .and_then(|d| d.custom_id.as_ref())
            .ok_or(LambdaInvokerError::NoCustomId)?;

        // Parse custom_id to extract the function name
        // Format: "function_name" or "function_name:arg1:arg2"
        let function_name = custom_id.split(':').next().unwrap_or(custom_id);
        let lambda_name = format!("discord-component-{}", function_name);
        info!("Invoking Lambda function: {}", lambda_name);

        self.invoke_lambda(&lambda_name, interaction).await
    }

    /// Invokes a Lambda function for a modal submit interaction
    async fn invoke_modal_lambda(
        &self,
        interaction: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        let custom_id = interaction
            .data
            .as_ref()
            .and_then(|d| d.custom_id.as_ref())
            .ok_or(LambdaInvokerError::NoCustomId)?;

        // Parse custom_id to extract the function name
        let function_name = custom_id.split(':').next().unwrap_or(custom_id);
        let lambda_name = format!("discord-modal-{}", function_name);
        info!("Invoking Lambda function: {}", lambda_name);

        self.invoke_lambda(&lambda_name, interaction).await
    }

    /// Invokes a Lambda function for autocomplete
    async fn invoke_autocomplete_lambda(
        &self,
        interaction: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        let command_name = interaction
            .data
            .as_ref()
            .and_then(|d| d.name.as_ref())
            .ok_or(LambdaInvokerError::NoCommandName)?;

        let function_name = format!("discord-autocomplete-{}", command_name);
        info!("Invoking Lambda function: {}", function_name);

        self.invoke_lambda(&function_name, interaction).await
    }

    /// Invokes a Lambda function with the given name and payload
    async fn invoke_lambda(
        &self,
        function_name: &str,
        payload: &Interaction,
    ) -> Result<Value, LambdaInvokerError> {
        let payload_json = serde_json::to_string(payload)?;
        let payload_blob = Blob::new(payload_json.as_bytes());

        info!("Invoking Lambda: {} with payload", function_name);

        let result = self
            .client
            .invoke()
            .function_name(function_name)
            .invocation_type(InvocationType::RequestResponse)
            .payload(payload_blob)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to invoke Lambda {}: {:?}", function_name, e);
                LambdaInvokerError::AwsSdkError(e.to_string())
            })?;

        // Check for function errors
        if let Some(function_error) = result.function_error() {
            error!(
                "Lambda function {} returned error: {}",
                function_name, function_error
            );
            return Err(LambdaInvokerError::AwsSdkError(format!(
                "Lambda function error: {}",
                function_error
            )));
        }

        // Parse the response
        if let Some(payload) = result.payload() {
            let response: Value = serde_json::from_slice(payload.as_ref())?;
            info!("Lambda {} invocation successful", function_name);
            Ok(response)
        } else {
            info!("Lambda {} returned no payload", function_name);
            Ok(Value::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discord::InteractionData;

    #[test]
    fn test_extract_function_name_from_custom_id() {
        let custom_id = "my_button:arg1:arg2";
        let function_name = custom_id.split(':').next().unwrap();
        assert_eq!(function_name, "my_button");
    }

    #[test]
    fn test_extract_function_name_simple() {
        let custom_id = "simple_button";
        let function_name = custom_id.split(':').next().unwrap();
        assert_eq!(function_name, "simple_button");
    }
}
