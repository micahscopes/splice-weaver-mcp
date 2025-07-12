#!/bin/bash

# Comprehensive smoke test runner for MCP ast-grep server and evaluation client

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    ((PASSED_TESTS++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((FAILED_TESTS++))
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    log_info "Running test: $test_name"
    ((TOTAL_TESTS++))
    
    if eval "$test_command"; then
        log_success "$test_name passed"
    else
        log_error "$test_name failed"
    fi
}

# Test 1: Build check
test_build() {
    log_info "Testing project build..."
    cargo build --all-targets
}

# Test 2: Unit tests
test_unit_tests() {
    log_info "Running unit tests..."
    cargo test --lib
}

# Test 3: MCP server smoke tests
test_mcp_server() {
    log_info "Running MCP server smoke tests..."
    cargo test --test mcp_server_smoke_tests
}

# Test 4: Evaluation client smoke tests
test_evaluation_client() {
    log_info "Running evaluation client smoke tests..."
    cargo test --test evaluation_client_smoke_tests
}

# Test 5: Integration tests
test_integration() {
    log_info "Running integration tests..."
    cargo test --test integration_tests
}

# Test 6: Binary smoke test
test_binaries() {
    log_info "Testing binary executables..."
    
    # Test MCP server binary
    timeout 5s cargo run --bin mcp-ast-grep &
    local mcp_pid=$!
    sleep 2
    
    if kill -0 $mcp_pid 2>/dev/null; then
        log_success "MCP server binary starts successfully"
        kill $mcp_pid 2>/dev/null || true
    else
        log_error "MCP server binary failed to start"
        return 1
    fi
    
    # Test evaluation client binary
    if ./target/debug/evaluation-client --help > /dev/null 2>&1; then
        log_success "Evaluation client binary works"
    else
        log_error "Evaluation client binary failed"
        return 1
    fi
}

# Test 7: Documentation tests
test_docs() {
    log_info "Testing documentation..."
    if cargo doc --no-deps > /dev/null 2>&1; then
        log_success "Documentation builds successfully"
    else
        log_error "Documentation build failed"
        return 1
    fi
}

# Test 8: Clippy lints
test_clippy() {
    log_info "Running clippy lints..."
    if cargo clippy --all-targets -- -D warnings; then
        log_success "Clippy passed"
    else
        log_warning "Clippy found issues (non-blocking)"
    fi
}

# Test 9: Format check
test_format() {
    log_info "Checking code formatting..."
    if cargo fmt --check; then
        log_success "Code is properly formatted"
    else
        log_warning "Code formatting issues found (non-blocking)"
    fi
}

# Test 10: Security audit
test_security() {
    log_info "Running security audit..."
    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            log_success "Security audit passed"
        else
            log_warning "Security audit found issues (review needed)"
        fi
    else
        log_warning "cargo-audit not installed, skipping security audit"
    fi
}

# Main test execution
main() {
    log_info "Starting comprehensive smoke tests for MCP ast-grep server"
    log_info "========================================================="
    
    # Ensure we're in the right directory
    cd "$(dirname "$0")/.."
    
    # Clean build first
    log_info "Cleaning previous builds..."
    cargo clean
    
    # Run all tests
    run_test "Build Check" "test_build"
    run_test "Unit Tests" "test_unit_tests"
    run_test "MCP Server Smoke Tests" "test_mcp_server"
    run_test "Evaluation Client Smoke Tests" "test_evaluation_client"
    run_test "Integration Tests" "test_integration"
    run_test "Binary Smoke Tests" "test_binaries"
    run_test "Documentation" "test_docs"
    run_test "Clippy Lints" "test_clippy"
    run_test "Format Check" "test_format"
    run_test "Security Audit" "test_security"
    
    # Summary
    log_info "========================================================="
    log_info "Test Summary:"
    log_info "Total tests: $TOTAL_TESTS"
    log_success "Passed: $PASSED_TESTS"
    log_error "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_success "All critical tests passed! 🎉"
        exit 0
    else
        log_error "Some tests failed. Please review the output above."
        exit 1
    fi
}

# Run main function
main "$@"