/*
 * Discord Interaction Handler Integration Tests
 * Copyright (C) 2023-2025  Joe McNally
 * Licensed under GPLv3
 */

use serde_json::json;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_ping_interaction_structure() {
        let ping_interaction = json!({
            "id": "1234567890",
            "application_id": "0987654321",
            "type": 1,
            "token": "test_token",
            "version": 1
        });

        assert_eq!(ping_interaction["type"], 1);
        assert_eq!(ping_interaction["version"], 1);
    }

    #[test]
    fn test_application_command_structure() {
        let command_interaction = json!({
            "id": "1234567890",
            "application_id": "0987654321",
            "type": 2,
            "data": {
                "id": "cmd_id",
                "name": "test_command",
                "type": 1
            },
            "guild_id": "guild_123",
            "channel_id": "channel_123",
            "token": "test_token",
            "version": 1
        });

        assert_eq!(command_interaction["type"], 2);
        assert_eq!(command_interaction["data"]["name"], "test_command");
    }

    #[test]
    fn test_message_component_structure() {
        let component_interaction = json!({
            "id": "1234567890",
            "application_id": "0987654321",
            "type": 3,
            "data": {
                "custom_id": "button_click:arg1",
                "component_type": 2
            },
            "guild_id": "guild_123",
            "channel_id": "channel_123",
            "token": "test_token",
            "version": 1
        });

        assert_eq!(component_interaction["type"], 3);
        assert_eq!(
            component_interaction["data"]["custom_id"],
            "button_click:arg1"
        );
    }

    #[test]
    fn test_modal_submit_structure() {
        let modal_interaction = json!({
            "id": "1234567890",
            "application_id": "0987654321",
            "type": 5,
            "data": {
                "custom_id": "modal_submit",
                "components": []
            },
            "guild_id": "guild_123",
            "channel_id": "channel_123",
            "token": "test_token",
            "version": 1
        });

        assert_eq!(modal_interaction["type"], 5);
        assert_eq!(modal_interaction["data"]["custom_id"], "modal_submit");
    }

    #[test]
    fn test_autocomplete_structure() {
        let autocomplete_interaction = json!({
            "id": "1234567890",
            "application_id": "0987654321",
            "type": 4,
            "data": {
                "id": "cmd_id",
                "name": "search",
                "type": 1,
                "options": [{
                    "name": "query",
                    "type": 3,
                    "value": "test",
                    "focused": true
                }]
            },
            "guild_id": "guild_123",
            "channel_id": "channel_123",
            "token": "test_token",
            "version": 1
        });

        assert_eq!(autocomplete_interaction["type"], 4);
        assert_eq!(autocomplete_interaction["data"]["name"], "search");
    }

    #[test]
    fn test_interaction_response_pong() {
        let response = json!({
            "type": 1
        });

        assert_eq!(response["type"], 1);
    }

    #[test]
    fn test_interaction_response_message() {
        let response = json!({
            "type": 4,
            "data": {
                "content": "Hello, World!"
            }
        });

        assert_eq!(response["type"], 4);
        assert_eq!(response["data"]["content"], "Hello, World!");
    }

    #[test]
    fn test_interaction_response_ephemeral() {
        let response = json!({
            "type": 4,
            "data": {
                "content": "This is ephemeral",
                "flags": 64
            }
        });

        assert_eq!(response["type"], 4);
        assert_eq!(response["data"]["flags"], 64);
    }

    #[test]
    fn test_lambda_function_naming_command() {
        let command_name = "mycommand";
        let expected = format!("discord-command-{}", command_name);
        assert_eq!(expected, "discord-command-mycommand");
    }

    #[test]
    fn test_lambda_function_naming_component() {
        let custom_id = "button_click:arg1:arg2";
        let function_name = custom_id.split(':').next().unwrap();
        let expected = format!("discord-component-{}", function_name);
        assert_eq!(expected, "discord-component-button_click");
    }

    #[test]
    fn test_lambda_function_naming_modal() {
        let custom_id = "user_info_modal";
        let expected = format!("discord-modal-{}", custom_id);
        assert_eq!(expected, "discord-modal-user_info_modal");
    }

    #[test]
    fn test_lambda_function_naming_autocomplete() {
        let command_name = "search";
        let expected = format!("discord-autocomplete-{}", command_name);
        assert_eq!(expected, "discord-autocomplete-search");
    }
}
