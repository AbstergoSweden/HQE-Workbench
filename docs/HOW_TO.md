# How-To Guide: Getting Started with HQE Workbench

Welcome to the HQE Workbench! This guide is designed to take you from "zero" to "hero," regardless of your experience level. We'll verify your system, install the tools, and run your first scan.

## Table of Contents

1. [Step 1: System Readiness](#step-1-system-readiness)
2. [Step 2: Installation](#step-2-installation)
3. [Step 3: Configuration](#step-3-configuration)
4. [Step 4: Your First Scan](#step-4-your-first-scan)
5. [Step 5: Using the Desktop App](#step-5-using-the-desktop-app)
6. [Troubleshooting](#troubleshooting)

---

## Step 1: System Readiness

Before we begin, let's make sure your Mac is ready.

1. **Check macOS Version**:
    - Click the Apple Menu () > About This Mac.
    - Ensure you are on **macOS Monterey (12.0)** or newer.

2. **Open Terminal**:
    - Press `Cmd + Space` (Spotlight).
    - Type `Terminal` and press Enter.

3. **Check for Git**:
    - Type `git --version` and press Enter.
    - If it says "git version ...", you're good. If not, a popup will ask to install it—say yes!

---

## Step 2: Installation

We have a script that handles everything for you (Rust, Node.js, Python dependencies).

1. **Clone the Repository** (Download the code):

    ```bash
    git clone https://github.com/AbstergoSweden/hqe-workbench.git
    cd hqe-workbench
    ```

2. **Run the Bootstrap Script**:

    ```bash
    ./scripts/bootstrap_macos.sh
    ```

    *Note: This might take a few minutes as it downloads compiler tools.*

3. **Build the Tool**:

    ```bash
    cargo build --release -p hqe
    ```

    - Once done, you have a working CLI tool in `target/release/hqe`.

---

## Step 3: Configuration

HQE Workbench uses "Profiles" to manage connection to AI providers (like OpenAI).

**To use Local-Only Mode (Recommended for high privacy):**
Title says it all—no configuration needed! You can skip to Step 4.

**To use OpenAI (Optional):**

1. Get your API Key from [OpenAI Platform](https://platform.openai.com/api-keys).
2. Run the configuration command:

    ```bash
    ./target/release/hqe config add "openai-main" --url "https://api.openai.com/v1" --key "sk-..." --model "gpt-4"
    ```

    *Your key is safely stored in the macOS Keychain, not in a text file.*

---

## Step 4: Your First Scan

Let's scan this repository itself to see how it works.

**Run a Local Scan:**

```bash
./target/release/hqe scan --repo . --local-only
```

**What happens?**

1. **Ingestion**: Finds all readable files (ignoring gitignore).
2. **Analysis**: Scans for "TODO" comments, fixmes, and basic heuristics.
3. **Reporting**: Generates a health report.

**View the Report:**
Look in the `hqe-output/` folder created in your current directory. You'll find a Markdown file (like `hqe_report_...md`) that you can open in any text editor.

---

## Step 5: Using the Desktop App

Prefer a GUI? We've got you covered.

1. **Start the App (Dev Mode)**:

    ```bash
    cd apps/workbench
    npm run tauri:dev
    ```

    *This will launch a native macOS window.*

2. **Navigate**:
    - **Dashboard**: See recent scans.
    - **New Scan**: Point to a folder on your Mac and click "Start".
    - **Settings**: Manage your AI profiles visually.

---

## Troubleshooting

### "Command not found: cargo"

- Restart your terminal. Rust updates your PATH, but it needs a restart to take effect.

### "Python not found"

- Ensure you have Python 3.11+. run `python3 --version`.

### "Permission denied"

- Try running commands with `sudo` only if absolutely necessary, but usually `chmod +x scripts/*.sh` fixes script permission issues.

### Still stuck?

Open an issue on [GitHub](https://github.com/AbstergoSweden/hqe-workbench/issues) or check `README.md` for community links.
