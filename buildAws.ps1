$env:GOOS = "linux"
$env:GOARCH = "amd64"
$env:CGO_ENABLED = "0"
go build -ldflags="-s -w" -o exe/eventHandler eventHandler.go
~\Go\Bin\build-lambda-zip.exe -o zips/eventHandler.zip exe/eventHandler