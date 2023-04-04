# Discord Interaction Handler

Discord Interaction Handler is a go project intended for deployment on AWS lambda to handle initial discord Interactions.
It will have to be expanded on to process commands.

## Installation

Note: Instructions assume that you are already familiar with [AWS Lambda](https://aws.amazon.com/lambda/) and APIs
1. Use git to clone the repository.
2. Run buildAWS.ps1.
3. Upload the zip to a go lambda function.
4. Ensure that the handler is set to "eventHandler".
5. Add the PUBLIC_KEY environment variable found in the General Information section on the Discord Developer Portal.
6. Link the lambda function to an API's HTTP POST method.
7. Enter the endpoint URL in the General Information section on the Discord Developer Portal.

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)