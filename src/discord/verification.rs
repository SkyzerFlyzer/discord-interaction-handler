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

use ed25519_dalek::{Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VerificationError {
    #[error("Missing signature header")]
    MissingSignature,
    #[error("Missing timestamp header")]
    MissingTimestamp,
    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),
    #[error("Invalid public key format: {0}")]
    InvalidPublicKeyFormat(String),
    #[error("Signature verification failed")]
    VerificationFailed,
}

/// Verifies a Discord interaction request signature using Ed25519
///
/// # Arguments
/// * `public_key` - The Discord application public key (hex encoded)
/// * `signature` - The signature from the x-signature-ed25519 header (hex encoded)
/// * `timestamp` - The timestamp from the x-signature-timestamp header
/// * `body` - The raw request body
///
/// # Returns
/// * `Ok(())` if the signature is valid
/// * `Err(VerificationError)` if verification fails
pub fn verify_discord_signature(
    public_key: &str,
    signature: &str,
    timestamp: &str,
    body: &str,
) -> Result<(), VerificationError> {
    // Decode the signature from hex
    let signature_bytes = hex::decode(signature)
        .map_err(|e| VerificationError::InvalidSignatureFormat(e.to_string()))?;

    if signature_bytes.len() != SIGNATURE_LENGTH {
        return Err(VerificationError::InvalidSignatureFormat(format!(
            "Expected {} bytes, got {}",
            SIGNATURE_LENGTH,
            signature_bytes.len()
        )));
    }

    // Decode the public key from hex
    let public_key_bytes = hex::decode(public_key)
        .map_err(|e| VerificationError::InvalidPublicKeyFormat(e.to_string()))?;

    if public_key_bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(VerificationError::InvalidPublicKeyFormat(format!(
            "Expected {} bytes, got {}",
            PUBLIC_KEY_LENGTH,
            public_key_bytes.len()
        )));
    }

    // Create the verifying key
    let verifying_key =
        VerifyingKey::from_bytes(public_key_bytes.as_slice().try_into().map_err(|_| {
            VerificationError::InvalidPublicKeyFormat("Failed to parse key".into())
        })?)
        .map_err(|e| VerificationError::InvalidPublicKeyFormat(e.to_string()))?;

    // Create the signature
    let signature = Signature::from_bytes(signature_bytes.as_slice().try_into().map_err(|_| {
        VerificationError::InvalidSignatureFormat("Failed to parse signature".into())
    })?);

    // Construct the message (timestamp + body)
    let message = format!("{}{}", timestamp, body);

    // Verify the signature
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| VerificationError::VerificationFailed)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_discord_signature_invalid_signature_length() {
        let public_key = "0".repeat(64); // Valid length public key
        let signature = "0".repeat(126); // Invalid length signature (63 bytes when decoded)
        let timestamp = "1234567890";
        let body = r#"{"type":1}"#;

        let result = verify_discord_signature(&public_key, &signature, timestamp, body);
        assert!(matches!(
            result,
            Err(VerificationError::InvalidSignatureFormat(_))
        ));
    }

    #[test]
    fn test_verify_discord_signature_invalid_public_key_length() {
        let public_key = "0".repeat(62); // Invalid length public key (31 bytes when decoded)
        let signature = "0".repeat(128); // Valid length signature
        let timestamp = "1234567890";
        let body = r#"{"type":1}"#;

        let result = verify_discord_signature(&public_key, &signature, timestamp, body);
        assert!(matches!(
            result,
            Err(VerificationError::InvalidPublicKeyFormat(_))
        ));
    }

    #[test]
    fn test_verify_discord_signature_invalid_hex() {
        let public_key = "ZZZZ"; // Invalid hex
        let signature = "0".repeat(128);
        let timestamp = "1234567890";
        let body = r#"{"type":1}"#;

        let result = verify_discord_signature(public_key, &signature, timestamp, body);
        assert!(matches!(
            result,
            Err(VerificationError::InvalidPublicKeyFormat(_))
        ));
    }
}
