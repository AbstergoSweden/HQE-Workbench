# Contributing to HQE Workbench

Thank you for your interest in contributing! We are building the future of autonomous engineering tools, and we'd love your help.

## Development Setup

1. **Fork & Clone**:

    ```bash
    git clone https://github.com/AbstergoSweden/HQE-Workbench.git
    cd HQE-Workbench
    ```

    If you are contributing from a fork, replace the clone URL with your fork URL.

2. **Bootstrap**:

    ```bash
    ./scripts/bootstrap_macos.sh
    ```

3. **Branching**:
    - Use `feat/` for new features.
    - Use `fix/` for bug fixes.
    - Use `docs/` for documentation updates.

## Pull Request Process

1. **Ensure Quality**: Run the full preflight check before pushing.

    ```bash
    npm run preflight
    ```

    This runs:
    - Rust tests & formatting (`cargo test`, `cargo fmt`)
    - TypeScript linting & testing (`npm run lint`, `npm test`)

    Optional (recommended) security audit:
    - `cargo audit`

2. **Commit Messages**: We follow [Conventional Commits](https://www.conventionalcommits.org/):
    - `feat(core): add new scanner`
    - `fix(ui): resolve button alignment`
    - `docs: update readme`

3. **Open PR**:
    - Target the `main` branch.
    - Fill out the PR template completely.
    - Link related issues (e.g., "Fixes #123").

## Style Guide

### Rust

- **Errors**: Use `thiserror` for libraries, `anyhow` for CLI/binaries.
- **Async**: Prefer `tokio` for async runtime.
- **Docs**: All public functions must have doc comments (`///`).

### TypeScript / React

- **Functional Components**: Use `React.FC` or simple functions.
- **State**: Use `zustand` for global state.
- **Styling**: Use Tailwind CSS (via `index.css`).
- **No `any`**: TypeScript strict mode is enabled.

## Community & Support

- **Issues**: Report bugs/features on GitHub.
- **Discussions**: Join our GitHub Discussions for deeper topics.

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
