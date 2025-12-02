# x402 Docker Integration Test Environment

This directory contains the Docker-based integration test setup for the x402 Rust implementation.

## Overview

The integration test environment includes:

- **Anvil**: Base node fork using Foundry's Anvil
- **Backend**: Axum server with x402 payment middleware
- **Facilitator**: Standalone facilitator service with Redis storage
- **Frontend**: Simple web application for testing payment flows
- **Redis**: Storage backend for facilitator nonce tracking

## Prerequisites

- Docker and Docker Compose installed
- Rust toolchain (for running tests)
- At least 4GB RAM available for Docker

## Quick Start

### 1. Start Services

```bash
# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f
```

### 2. Wait for Services to be Ready

Services will start in order:
1. Anvil (Base node fork)
2. Redis
3. Facilitator (depends on Redis and Anvil)
4. Backend (depends on Facilitator and Anvil)
5. Frontend (depends on Backend)

You can check health endpoints:
- Backend: http://localhost:4021/health
- Facilitator: http://localhost:4020/health
- Frontend: http://localhost:3000

### 3. Run Integration Tests

```bash
# Run Docker integration tests
cargo test --test docker_integration_test --features axum,redis -- --nocapture

# Run specific test
cargo test --test docker_integration_test test_end_to_end_payment_flow --features axum,redis -- --nocapture
```

### 4. Access Frontend

Open http://localhost:3000 in your browser to access the test frontend.

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

### Frontend

- **Port**: 3000
- **Purpose**: Web interface for testing payment flows
- **Features**:
  - Test protected endpoints
  - View payment requirements
  - Display payment status

### Redis

- **Port**: 6379
- **Purpose**: Nonce storage for facilitator
- **Persistence**: Enabled with AOF

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and modify as needed:

```bash
cp integration-test/.env.example integration-test/.env
```

Key variables:
- `ANVIL_FORK_URL`: Base Sepolia RPC URL for forking
- `FACILITATOR_URL`: Facilitator service URL
- `RPC_URL`: Anvil RPC URL
- `NETWORK`: Network name (base-sepolia)

### Docker Compose

Modify `docker-compose.yml` to adjust:
- Port mappings
- Resource limits
- Service dependencies
- Health check intervals

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

### Services Not Starting

```bash
# Check logs
docker-compose logs

# Restart services
docker-compose restart

# Rebuild images
docker-compose build --no-cache
```

### Port Conflicts

If ports are already in use, modify `docker-compose.yml`:

```yaml
ports:
  - "4022:4021"  # Change host port
```

### Anvil Fork Issues

If Anvil fails to fork:

1. Check internet connection
2. Verify `ANVIL_FORK_URL` is accessible
3. Try a different fork block number

### Test Failures

1. Ensure all services are healthy
2. Check service logs: `docker-compose logs`
3. Verify network connectivity between services
4. Check test account has sufficient balance

## Development

### Building Images Locally

```bash
# Build specific service
docker-compose build backend

# Build all services
docker-compose build
```

### Running Tests Locally

```bash
# Start services
docker-compose up -d

# Run tests
cargo test --test docker_integration_test --features axum,redis

# Stop services
docker-compose down
```

### Modifying Services

1. Make code changes
2. Rebuild image: `docker-compose build <service>`
3. Restart service: `docker-compose restart <service>`

## CI/CD

The GitHub Actions workflow (`.github/workflows/docker-integration.yml`) automatically:

1. Starts Docker services
2. Waits for health checks
3. Runs integration tests
4. Collects logs on failure
5. Cleans up services

## Architecture

```
┌─────────┐     ┌──────────┐     ┌─────────────┐
│ Frontend│────▶│ Backend │────▶│ Facilitator│
└─────────┘     └────┬────┘     └──────┬──────┘
                     │                 │
                     │                 │
                ┌────▼────┐      ┌─────▼────┐
                │  Anvil  │      │  Redis   │
                │  (RPC)  │      │ (Storage)│
                └─────────┘      └──────────┘
```

## Security Notes

⚠️ **Warning**: This setup is for testing only!

- Test private keys are exposed in code
- No authentication on services
- Anvil uses default accounts
- Do NOT use in production

## Cleanup

```bash
# Stop and remove containers
docker-compose down

# Remove volumes (clears Redis data)
docker-compose down -v

# Remove images
docker-compose down --rmi all
```

## Next Steps

- Add more test scenarios
- Implement USDC contract deployment
- Add performance benchmarks
- Create load testing scenarios

