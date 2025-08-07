# Build stage
FROM golang:1.22-alpine AS builder

WORKDIR /app

# Copy go mod files
COPY go.mod go.sum ./
RUN go mod download

# Copy source code
COPY . .

# Build the binary
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o codeowners-validation ./cmd

# Runtime stage
FROM alpine:latest

RUN apk --no-cache add ca-certificates git
WORKDIR /root/

# Copy the binary from builder stage to system path
COPY --from=builder /app/codeowners-validation /usr/local/bin/codeowners-validation
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /github/workspace

ENTRYPOINT ["/entrypoint.sh"]

