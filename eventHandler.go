/*
 * Discord Interaction Handler is a go project intended for deployment on AWS lambda to handle initial discord Interactions.
 *     Copyright (C) 2023  Joe McNally
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

package main

import (
	"context"
	"crypto/ed25519"
	"encoding/hex"
	"encoding/json"
	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/bwmarrin/discordgo"
	"os"
)

// Load the public ket from environment variables
var publicKey = os.Getenv("PUBLIC_KEY")

// HandleRequest A function to handle and process the discord Interaction event as per discord guidelines.
// Verifies that the signature is correct first and errors with a 401 if it isn't.
// Checks if it's a ping request and returns '{"type": 1}' if it is.
// Checks if the type is valid for a command and returns 400 if it isn't
// Process commands
func HandleRequest(_ context.Context, event events.APIGatewayProxyRequest) (interface{}, error) {
	headers := map[string]string{
		"Content-Type": "application/json",
	}
	if !verifySignature(event) {
		responseBody := generateResponseBody([]byte(`{"message":"invalid request signature"}`))
		response := events.APIGatewayProxyResponse{StatusCode: 401,
			Headers: headers,
			Body:    responseBody}
		return response, nil
	}
	jsonBody := getBodyJson([]byte(event.Body))
	if checkPing(jsonBody) {
		responseBody := generateResponseBody([]byte(`{"type": 1}`))
		response := events.APIGatewayProxyResponse{StatusCode: 200,
			Headers: headers,
			Body:    responseBody}
		return response, nil
	}
	if !(jsonBody.Type >= 4 && jsonBody.Type <= 9) {
		responseBody := generateResponseBody([]byte(`{"message":"bad request"}`))
		response := events.APIGatewayProxyResponse{StatusCode: 400,
			Headers: headers,
			Body:    responseBody}
		return response, nil
	}
	responseData := discordgo.InteractionResponseData{
		TTS:             false,
		Content:         "Command not yet implemented.",
		Components:      nil,
		Embeds:          nil,
		AllowedMentions: nil,
		Files:           nil,
		Flags:           0,
		Choices:         nil,
		CustomID:        "",
		Title:           "",
	}
	responseBody := discordgo.InteractionResponse{
		Type: 4,
		Data: &responseData,
	}
	body, _ := json.Marshal(responseBody)
	response := events.APIGatewayProxyResponse{StatusCode: 200,
		Headers: headers,
		Body:    string(body)}
	return response, nil
}

//  verifySignature Extracts the raw body from the event and then checks it against the verify function
func verifySignature(event events.APIGatewayProxyRequest) bool {
	raw := event.Body
	return verify([]byte(raw), event.Headers, publicKey)
}

// verify Checks that the signature is valid.
// First checks the signature header is set, if not returns false.
// Checks that the signature is valid and of correct size if not returns false.
// Checks the timestamp is set, if not returns false.
// Checks that the public key is valid, if not returns false.
// Performs an ed25519.Verify check on the entire message
func verify(rawBody []byte, headers map[string]string, publicKey string) bool {
	signature := headers["x-signature-ed25519"]
	if signature == "" {
		return false
	}

	sig, err := hex.DecodeString(signature)
	if err != nil {
		return false
	} else if len(sig) != ed25519.SignatureSize {
		return false
	}

	timestamp := headers["x-signature-timestamp"]
	if timestamp == "" {
		return false
	}

	keyBytes, err := hex.DecodeString(publicKey)
	if err != nil {
		return false
	}

	key := ed25519.PublicKey(keyBytes)
	if len(key) != 32 {
		return false
	}

	msg := []byte(timestamp + string(rawBody))
	return ed25519.Verify(key, msg, sig)
}

// getBodyJson converts the body to a discordgo.Interaction
func getBodyJson(body []byte) discordgo.Interaction {
	interaction := discordgo.Interaction{}
	err := json.Unmarshal(body, &interaction)
	if err != nil {
		return discordgo.Interaction{}
	}
	return interaction

}

// generateResponseBody Converts a message in bytes to a json string
func generateResponseBody(message []byte) string {
	jsonData := make(map[string]interface{})
	err := json.Unmarshal(message, &jsonData)
	if err != nil {
		return ""
	}
	response, _ := json.Marshal(jsonData)
	return string(response)

}

// checkPing Checks if the request sent is a ping request.
func checkPing(body discordgo.Interaction) bool {
	if body.Type == 1 {
		return true
	}
	return false
}

// main entry point of the program that AWS Lambda will call
func main() {
	lambda.Start(HandleRequest)
}
