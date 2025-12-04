# x402 Docker Integration Test Environment

This directory contains the Docker-based integration test setup for the x402 Rust implementation.

## Overview

The integration test environment uses a single all-in-one Docker container that includes:

- **Anvil**: Base node fork using Foundry's Anvil
- **Redis**: Storage backend for facilitator nonce tracking
- **Backend**: Axum server with x402 payment middleware
- **Facilitator**: Standalone facilitator service with Redis storage

## Prerequisites

- Docker installed
- Rust toolchain (for running tests)
- At least 2GB RAM available for Docker

## Quick Start

### 1. Build the All-in-One Container

```bash
# Build the container
docker build -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .

# Or with BuildKit for faster builds
DOCKER_BUILDKIT=1 docker build -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .
```

### 2. Start the Container

```bash
# Start the container
docker run -d \
  --name x402-test \
  -p 8545:8545 \
  -p 6379:6379 \
  -p 4020:4020 \
  -p 4021:4021 \
  x402-all-in-one

# Check container status
docker ps | grep x402-test

# View logs
docker logs -f x402-test
```

### 3. Wait for Services to be Ready

The container will start all services automatically. Wait for the log message:
```
All services are ready!
```

You can also check health endpoints:
- Backend: http://localhost:4021/health
- Facilitator: http://localhost:4020/health
- Anvil: http://localhost:8545 (RPC endpoint)

### 4. Run Integration Tests

```bash
# Run Docker integration tests
cargo test --test docker_integration_test --features axum,redis -- --nocapture

# Run specific test
cargo test --test docker_integration_test test_end_to_end_payment_flow --features axum,redis -- --nocapture
```

### 5. Stop the Container

```bash
# Stop the container
docker stop x402-test

# Remove the container
docker rm x402-test
```

## Service Details

### Anvil (Base Node Fork)

- **Port**: 8545
- **Purpose**: Local Base Sepolia fork for testing
- **Pre-funded accounts**: 10 accounts with 10,000 ETH each
- **Fork URL**: Configurable via `ANVIL_FORK_URL` environment variable

Default test account:
- Address: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- Private Key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`

### Backend

- **Port**: 4021
- **Purpose**: Axum server with x402 payment middleware
- **Endpoints**:
  - `GET /health` - Health check (no payment required)
  - `GET /joke` - Premium joke API (payment required)
  - `GET /api/data` - Premium data API (payment required)
  - `GET /test` - Test endpoint (payment required)
  - `GET /download` - File download (payment required)

### Facilitator

- **Port**: 4020
- **Purpose**: Payment verification and settlement
- **Storage**: Redis backend
- **Endpoints**:
  - `POST /verify` - Verify payment authorization
  - `POST /settle` - Settle verified payment
  - `GET /supported` - Get supported payment schemes
  - `GET /health` - Health check

### Redis

- **Port**: 6379
- **Purpose**: Nonce storage for facilitator
- **Persistence**: Enabled with AOF

## Configuration

### Environment Variables

You can customize the container behavior with environment variables:

```bash
docker run -d \
  --name x402-test \
  -p 8545:8545 \
  -p 6379:6379 \
  -p 4020:4020 \
  -p 4021:4021 \
  -e ANVIL_FORK_URL=https://sepolia.base.org \
  -e ANVIL_FORK_BLOCK=latest \
  x402-all-in-one
```

Key variables:
- `ANVIL_FORK_URL`: Base Sepolia RPC URL for forking (default: https://sepolia.base.org)
- `ANVIL_FORK_BLOCK`: Specific block number to fork from (default: latest)
- `NETWORK`: Network name (default: base-sepolia)

## Testing Scenarios

### 1. Health Checks

Verify all services are running:

```bash
curl http://localhost:4021/health
curl http://localhost:4020/health
```

### 2. Payment Flow

1. Request protected endpoint (should get 402)
2. Parse payment requirements
3. Create signed payment payload
4. Retry request with payment header
5. Verify success and settlement

### 3. Error Scenarios

- Expired authorization
- Insufficient funds
- Invalid signature
- Replay attack (duplicate nonce)

## Troubleshooting

### Container Not Starting

```bash
# Check logs
docker logs x402-test

# Check container status
docker ps -a | grep x402-test

# Restart container
docker restart x402-test
```

### Port Conflicts

If ports are already in use, use different host ports:

```bash
docker run -d \
  --name x402-test \
  -p 8546:8545 \    # Changed from 8545
  -p 6380:6379 \    # Changed from 6379
  -p 4022:4020 \    # Changed from 4020
  -p 4023:4021 \    # Changed from 4021
  x402-all-in-one
```

Then update test URLs in `tests/docker_integration_test.rs`:
```rust
const BACKEND_URL: &str = "http://localhost:4023";
const FACILITATOR_URL: &str = "http://localhost:4022";
```

### Anvil Fork Issues

If Anvil fails to fork:

1. Check internet connection
2. Verify `ANVIL_FORK_URL` is accessible
3. Try a different fork block number
4. Check container logs: `docker logs x402-test`

### Test Failures

1. Ensure container is running: `docker ps | grep x402-test`
2. Check service logs: `docker logs x402-test`
3. Verify health endpoints are responding
4. Check test account has sufficient balance

## Development

### Building the Container

```bash
# Standard build
docker build -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .

# Build with BuildKit cache (faster rebuilds)
DOCKER_BUILDKIT=1 docker build -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .

# Build without cache (clean build)
docker build --no-cache -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .
```

### Running Tests Locally

```bash
# Start container
docker run -d --name x402-test \
  -p 8545:8545 -p 6379:6379 -p 4020:4020 -p 4021:4021 \
  x402-all-in-one

# Wait for services to be ready (check logs)
docker logs -f x402-test

# Run tests
cargo test --test docker_integration_test --features axum,redis

# Stop container
docker stop x402-test
docker rm x402-test
```

### Modifying Services

1. Make code changes
2. Rebuild container: `docker build -f integration-test/Dockerfile.all-in-one -t x402-all-in-one .`
3. Stop old container: `docker stop x402-test && docker rm x402-test`
4. Start new container: `docker run -d --name x402-test -p 8545:8545 -p 6379:6379 -p 4020:4020 -p 4021:4021 x402-all-in-one`

## CI/CD

The GitHub Actions workflow (`.github/workflows/docker-integration.yml`) should:

1. Build the all-in-one container
2. Start the container
3. Wait for health checks
4. Run integration tests
5. Collect logs on failure
6. Clean up container

## Architecture

```
┌─────────────────────────────────────────┐
│         All-in-One Container            │
│                                         │
│  ┌─────────┐     ┌──────────┐          │
│  │ Backend │────▶│Facilitator│         │
│  └────┬────┘     └──────┬───┘          │
│       │                 │               │
│  ┌────▼────┐      ┌─────▼────┐         │
│  │  Anvil  │      │  Redis   │         │
│  │  (RPC)  │      │ (Storage)│         │
│  └─────────┘      └──────────┘          │
│                                         │
└─────────────────────────────────────────┘
```

## Advantages of All-in-One Approach

- **Simplicity**: Single container to manage
- **Fast Startup**: All services start together
- **Resource Efficient**: Shared base image
- **Easy Testing**: One command to start everything
- **CI/CD Friendly**: Simpler workflow setup

## Security Notes

⚠️ **Warning**: This setup is for testing only!

- Test private keys are exposed in code
- No authentication on services
- Anvil uses default accounts
- Do NOT use in production

## Cleanup

```bash
# Stop and remove container
docker stop x402-test
docker rm x402-test

# Remove image
docker rmi x402-all-in-one

# Clean up all Docker resources (use with caution)
docker system prune -a
```

## Next Steps

- Add more test scenarios
- Implement USDC contract deployment
- Add performance benchmarks
- Create load testing scenarios
