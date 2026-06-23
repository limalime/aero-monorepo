# Circuits Tests

Test functions are defined inline in the source modules under `src/` using Noir's `#[test]` attribute. Additional focused test modules (`test_utils.nr`, `test_integration.nr`) live alongside the source and are declared in `main.nr`.

## Running Tests

```bash
nargo test
```

## Test Coverage

- **src/utils.nr** — `#[test]` functions in `test_utils.nr`: byte conversion, hashing, loan computation, validation helpers
- **src/invoice.nr** — inline `#[test]` functions: validation, commitment determinism, collision resistance
- **src/zktls.nr** — inline `#[test]` functions: provider lookup, commitment integrity, zero-provider rejection
- **src/lending.nr** — inline `#[test]` functions: param validation, LTV enforcement, loan computation
- **src/main.nr** — inline `#[test]` functions: happy path, nullifier determinism, nullifier uniqueness
- **src/test_integration.nr** — cross-module integration: multi-currency, boundary values, multiple providers, near-expiry