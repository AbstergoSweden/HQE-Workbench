# Contributing to HQE Workbench

Thank you for your interest in contributing! This project follows the **High Quality Engineering (HQE)** protocol.

## Development Setup

1. **Fork and Clone**:

   ```bash
   git clone https://github.com/YOUR_USERNAME/hqe-workbench.git
   cd hqe-workbench
   ```

2. **Install Dependencies**:

   ```bash
   ./scripts/bootstrap_macos.sh
   # OR manually:
   # rustup update stable
   # npm install -g yarn
   # cd apps/workbench && npm ci
   ```

3. **Create Branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

## Workflow

1. **Make Changes**:
   - Backend logic goes in `crates/`.
   - CLI commands go in `cli/hqe/src/commands/`.
   - UI components go in `apps/workbench/src/components/`.

2. **Verify Changes**:
   - **Test**: `cargo test --workspace` AND `cd apps/workbench && npm test`
   - **Lint**: `cargo clippy` AND `npm run lint` (in frontend)
   - **Format**: `cargo fmt` AND `npm run format` (if available)

3. **Commit**:
   We follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(core): add new scanner`
   - `fix(ui): resolve button alignment`
   - `docs: update readme`

4. **Push and PR**:
   - Push to your fork.
   - Open a Pull Request against `main`.
   - Ensure CI checks pass.

## Code Standards

- **Rust**: Follow standard Rust idioms (clippy is your friend).
- **TypeScript**: Strict mode enabled. No `any`.
- **Testing**: New features generally require unit tests.
- **Security**: No hardcoded secrets. Use the `secrecy` crate for sensitive data.

## License

By contributing, you agree that your contributions will be licensed under the project's MIT License.
