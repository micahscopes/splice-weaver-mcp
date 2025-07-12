#!/bin/bash

# LLM Performance Benchmark Runner
# This script demonstrates how to use the benchmark framework

set -e

echo "🚀 LLM Performance Benchmark Framework"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust first."
    exit 1
fi

# Build the project
print_status "Building benchmark framework..."
cargo build --release --bin benchmark-runner

if [ $? -ne 0 ]; then
    print_error "Failed to build benchmark framework"
    exit 1
fi

print_success "Build completed successfully!"

# Generate default config if it doesn't exist
if [ ! -f "benchmark_config.yaml" ]; then
    print_status "Generating default benchmark configuration..."
    ./target/release/benchmark-runner --generate-config
    print_success "Default configuration generated at benchmark_config.yaml"
    print_warning "Please edit benchmark_config.yaml to configure your LLM endpoints"
fi

# Show available options
echo ""
echo "📋 Available Commands:"
echo "====================="
echo ""
echo "1. List test scenarios:"
echo "   ./target/release/benchmark-runner --list-scenarios"
echo ""
echo "2. Validate configuration:"
echo "   ./target/release/benchmark-runner --dry-run"
echo ""
echo "3. Run specific scenarios:"
echo "   ./target/release/benchmark-runner --scenario \"Basic Function Search\" --scenario \"Variable Refactoring\""
echo ""
echo "4. Run specific models:"
echo "   ./target/release/benchmark-runner --model \"gpt-3.5-turbo\" --model \"gpt-4\""
echo ""
echo "5. Run quick benchmark (5 iterations):"
echo "   ./target/release/benchmark-runner --iterations 5"
echo ""
echo "6. Run full benchmark suite:"
echo "   ./target/release/benchmark-runner"
echo ""
echo "7. Run Criterion benchmarks:"
echo "   cargo bench"
echo ""

# Check if LLM server is running
print_status "Checking LLM server availability..."
if curl -s -f "http://localhost:1234/v1/models" > /dev/null 2>&1; then
    print_success "LLM server is running at http://localhost:1234"
else
    print_warning "LLM server not detected at http://localhost:1234"
    print_warning "Please start your LLM server (e.g., LM Studio, vLLM) before running benchmarks"
fi

# Demonstrate different benchmark modes
echo ""
echo "🔄 Running demonstration benchmarks..."
echo "======================================"

# Quick validation
print_status "Validating configuration..."
./target/release/benchmark-runner --dry-run

if [ $? -eq 0 ]; then
    print_success "Configuration validation passed!"
    
    # List scenarios
    print_status "Available test scenarios:"
    ./target/release/benchmark-runner --list-scenarios
    
    # Ask user if they want to run a quick benchmark
    echo ""
    read -p "Run a quick benchmark with 3 iterations? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Running quick benchmark..."
        ./target/release/benchmark-runner --iterations 3 --scenario "Basic Function Search"
        
        if [ $? -eq 0 ]; then
            print_success "Quick benchmark completed! Check benchmark_results/ for detailed results."
        else
            print_error "Quick benchmark failed. Check your LLM server configuration."
        fi
    fi
else
    print_error "Configuration validation failed. Please check benchmark_config.yaml"
fi

echo ""
echo "📊 Benchmark Results:"
echo "===================="
echo ""
echo "Results will be saved in the following formats:"
echo "• JSON: benchmark_results/benchmark_results.json"
echo "• CSV: benchmark_results/benchmark_summary.csv"
echo "• HTML: benchmark_results/benchmark_report.html"
echo ""
echo "For Criterion benchmarks, results will be in:"
echo "• HTML: target/criterion/report/index.html"
echo ""

echo "🎯 Usage Tips:"
echo "============="
echo ""
echo "1. Configure your LLM endpoints in benchmark_config.yaml"
echo "2. Start with a small number of iterations (--iterations 5) for testing"
echo "3. Use --dry-run to validate configuration before running full benchmarks"
echo "4. Filter scenarios and models to focus on specific tests"
echo "5. Use cargo bench for statistical analysis with Criterion"
echo "6. Check benchmark_results/ for detailed performance reports"
echo ""

print_success "Benchmark framework is ready to use!"