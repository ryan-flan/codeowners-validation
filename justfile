# Build commands for CODEOWNERS validation

# Default command - build the binary
default: build

# Install dependencies
deps:
    go mod download

# Build development binary
build:
    go build -o codeowners-validation ./cmd

# Build optimized release binary
build-release:
    go build -ldflags="-s -w" -o codeowners-validation ./cmd

# Build for all platforms
build-all: clean
    mkdir -p dist
    GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o dist/codeowners-validation-linux-amd64 ./cmd
    GOOS=darwin GOARCH=amd64 go build -ldflags="-s -w" -o dist/codeowners-validation-darwin-amd64 ./cmd
    GOOS=darwin GOARCH=arm64 go build -ldflags="-s -w" -o dist/codeowners-validation-darwin-arm64 ./cmd
    GOOS=windows GOARCH=amd64 go build -ldflags="-s -w" -o dist/codeowners-validation-windows-amd64.exe ./cmd

# Run tests
test:
    go test ./...

# Run tests with race detection and coverage
test-full:
    go test -v -race -coverprofile=coverage.out ./...

# Run linting and formatting checks
lint:
    go vet ./...
    gofmt -s -l .

# Format code
fmt:
    gofmt -s -w .

# View test coverage in browser
coverage: test-full
    go tool cover -html=coverage.out

# Clean build artifacts
clean:
    rm -f codeowners-validation
    rm -rf dist/
    rm -f coverage.out

# Run all quality checks (lint, test, build)
check: lint test build

# Setup development environment (if using flox)
setup:
    @echo "Activating flox environment..."
    @echo "Run: flox activate"
    @echo "Then run: just deps"

# Run the tool with default CODEOWNERS file
run:
    ./codeowners-validation --path .github/CODEOWNERS

# Run the tool with all checks
run-all:
    ./codeowners-validation --path .github/CODEOWNERS --checks all

# Show available commands
help:
    @just --list