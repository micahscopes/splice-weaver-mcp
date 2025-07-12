# Testing Guide - Cargo-Native Approach

This document describes the comprehensive testing strategy for our MCP ast-grep server and Rust evaluation client using Cargo's built-in test system.

## Why Cargo Tests Only?

We use Cargo's native test system instead of shell scripts because:
- ✅ **Integrated**: Built into the Rust toolchain
- ✅ **Fast**: Parallel execution and intelligent caching
- ✅ **Discoverable**: Standard `cargo test` commands
- ✅ **Maintainable**: No external script dependencies
- ✅ **CI/CD Friendly**: Works everywhere Rust works

## Test Structure

```
tests/
├── smoke_tests.rs          # Essential functionality verification
├── integration_tests.rs    # End-to-end workflow testing
└── test_helpers.rs        # Shared utilities and helpers

src/
└── evaluation_client.rs   # Unit tests embedded in the module
```

## Quick Verification

Run all tests with a single command:

```bash
cargo test
```

For specific test categories:

```bash
# Run only smoke tests
cargo test smoke

# Run only integration tests  
cargo test integration

# Run a specific test
cargo test smoke_evaluation_client

# Run with output visible
cargo test -- --nocapture
```

## Test Categories

### 1. Smoke Tests (`cargo test smoke`)

Essential functionality verification that should always pass:

- **Build & Binaries**: Verify executables work
- **MCP Server**: Basic protocol compliance and startup
- **Tool Discovery**: Server lists expected tools  
- **Evaluation Client**: Core functionality works
- **End-to-End**: Simple file processing workflow

```bash
cargo test smoke_comprehensive
```

### 2. Integration Tests (`cargo test integration`)

Higher-level workflow validation:

- **Evaluation Suite**: Test case management
- **Full Workflow**: Client → MCP → Tools → Response
- **Test Case Structure**: Default test cases are well-formed

```bash
cargo test test_evaluation_client_full_workflow
```

### 3. Unit Tests (`cargo test --lib`)

Module-level functionality testing embedded in source files.

## Success Criteria

Tests verify:
- ✅ **Core functionality**: All primary features work
- ✅ **Error handling**: Graceful failure modes
- ✅ **Integration**: Components work together
- ✅ **Compatibility**: Standard Rust/Cargo behavior
- ✅ **Performance**: Reasonable execution times

**Quick verification**: `cargo test` should complete successfully, indicating the entire system is working correctly.