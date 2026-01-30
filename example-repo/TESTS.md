# Test Suite for HQE Example Repository

This repository contains a comprehensive set of tests to validate the functionality of the HQE Workbench when running with the Venice AI API.

## Test Categories

### 1. Security Tests
- SQL injection detection
- Cross-site scripting (XSS) prevention
- Authentication bypass prevention
- Authorization checks
- Input validation
- Path traversal protection
- Insecure deserialization prevention

### 2. Functional Tests
- Repository scanning functionality
- Report generation
- Configuration management
- API integration
- File processing
- Data transformation

### 3. Performance Tests
- Large repository handling
- Concurrent scan operations
- Memory usage optimization
- Response time measurements

### 4. Integration Tests
- End-to-end workflows
- API communication
- Database operations
- File system operations

## Test Execution

The tests can be executed using the standard Node.js testing framework:

```bash
npm test
```

Or for specific test suites:

```bash
npm run test:security
npm run test:functional
npm run test:performance
npm run test:integration
```

## Test Coverage

The test suite aims for 90%+ code coverage, with particular emphasis on security-critical components.