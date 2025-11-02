#!/bin/bash
# Local CI test script - mimics GitHub Actions CI workflow

set -e  # Exit on error

echo "🧪 Testing x402 CI workflow locally..."
echo ""

# Test matrix configurations
RUST_VERSIONS=("stable" "beta" "nightly")
FEATURES=("default" "axum" "actix-web" "warp")

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_config() {
    local rust_version=$1
    local feature=$2
    
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Testing: Rust ${rust_version} with feature '${feature}'${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Install/switch toolchain
    echo "📦 Installing Rust ${rust_version}..."
    rustup toolchain install ${rust_version}
    rustup component add clippy rustfmt --toolchain ${rust_version}
    
    # Run tests
    echo "🧪 Running tests..."
    if [ "$feature" = "default" ]; then
        cargo +${rust_version} test --all-features
    else
        cargo +${rust_version} test --features ${feature}
    fi
    
    # Run clippy
    echo "🔍 Running clippy..."
    cargo +${rust_version} clippy --all-features -- -D warnings
    
    # Run fmt check
    echo "📝 Running fmt check..."
    cargo +${rust_version} fmt --all -- --check
    
    echo -e "${GREEN}✅ Passed: Rust ${rust_version} with feature '${feature}'${NC}"
    echo ""
}

# Quick test - just stable with default features
quick_test() {
    echo "⚡ Quick test mode (stable + default features)"
    test_config "stable" "default"
}

# Full test - all combinations
full_test() {
    echo "🔬 Full test mode (all Rust versions × all features)"
    for rust in "${RUST_VERSIONS[@]}"; do
        for feature in "${FEATURES[@]}"; do
            test_config "$rust" "$feature"
        done
    done
}

# Parse arguments
case "${1:-quick}" in
    quick)
        quick_test
        ;;
    full)
        full_test
        ;;
    stable)
        test_config "stable" "${2:-default}"
        ;;
    beta)
        test_config "beta" "${2:-default}"
        ;;
    nightly)
        test_config "nightly" "${2:-default}"
        ;;
    *)
        echo "Usage: $0 [quick|full|stable|beta|nightly] [feature]"
        echo ""
        echo "Examples:"
        echo "  $0                    # Quick test (stable + default)"
        echo "  $0 quick              # Quick test"
        echo "  $0 full               # Full test (all combinations)"
        echo "  $0 stable axum        # Test stable with axum feature"
        echo "  $0 nightly default    # Test nightly with all features"
        exit 1
        ;;
esac

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All tests completed successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

