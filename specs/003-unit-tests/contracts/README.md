# Test Contracts

This directory contains example test structures that serve as "contracts" for how unit tests should be organized and implemented for the windsurf-forecast project.

## Purpose

These contract files demonstrate:
- Test organization patterns
- Mock data structure examples
- Assertion helper patterns
- Test fixture creation
- Error testing approaches

## Contract Files

1. **args_test_contract.rs** - CLI argument validation test patterns
2. **provider_test_contract.rs** - Provider transformation test patterns with mocks
3. **config_test_contract.rs** - Configuration file and precedence test patterns
4. **timezone_test_contract.rs** - Timezone conversion test patterns
5. **github-workflow-tests.yml** - GitHub Actions CI/CD configuration for automated testing and coverage

## Usage

These are **example contracts**, not executable code. They show the expected structure and patterns that should be followed when implementing actual tests in the `tests/` directory.

Key principles demonstrated:
- Clear Arrange-Act-Assert structure
- Single responsibility per test
- Descriptive test names
- Proper mock isolation
- Environment variable safety
- Coverage of success and error paths