# Agent Prompts MCP Server

<div align="center">

<img src="assets/logo.png" alt="Agent Prompts MCP Server Logo" width="200" />

[![npm version](https://img.shields.io/npm/v/agent-prompts.svg?style=for-the-badge&logo=npm&color=0066cc)](https://www.npmjs.com/package/agent-prompts)
<a href="cursor://anysphere.cursor-deeplink/mcp/install?name=agent-prompts&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsImFnZW50LXByb21wdHNAbGF0ZXN0Il19"><img src="https://cursor.com/deeplink/mcp-install-dark.png" alt="Add agent-prompts MCP server to Cursor" height="28" /></a>
[![License: MIT](https://img.shields.io/badge/License-MIT-00ff88.svg?style=for-the-badge&logo=opensource)](https://opensource.org/licenses/MIT)

**Hot-reloadable prompts, structured reasoning, and chain workflows for your AI assistant.**

[Quick Start](#quick-start) • [What You Get](#what-you-get) • [Syntax](#syntax-reference) • [Docs](#documentation)

</div>

---

## Why

Stop copy-pasting prompts. This server turns your prompt library into a programmable engine:

- **Version Control** — Prompts are YAML + templates in git. Track changes, review diffs.
- **Hot Reload** — Edit a template, run it immediately. No restarts.
- **Structured Execution** — Parses operators, injects methodology, enforces quality gates.

---

## How It Works

```mermaid
%%{init: {'theme': 'neutral', 'themeVariables': {'background':'#0b1224','primaryColor':'#e2e8f0','primaryBorderColor':'#1f2937','primaryTextColor':'#0f172a','lineColor':'#94a3b8','fontFamily':'"DM Sans","Segoe UI",sans-serif','fontSize':'14px','edgeLabelBackground':'#0b1224'}}}%%
flowchart TB
    classDef actor fill:#0f172a,stroke:#cbd5e1,stroke-width:1.5px,color:#f8fafc;
    classDef server fill:#111827,stroke:#fbbf24,stroke-width:1.8px,color:#f8fafc;
    classDef process fill:#e2e8f0,stroke:#1f2937,stroke-width:1.6px,color:#0f172a;
    classDef client fill:#f4d0ff,stroke:#a855f7,stroke-width:1.6px,color:#2e1065;
    classDef clientbg fill:#1a0a24,stroke:#a855f7,stroke-width:1.8px,color:#f8fafc;
    classDef decision fill:#fef3c7,stroke:#f59e0b,stroke-width:1.6px,color:#78350f;

    linkStyle default stroke:#94a3b8,stroke-width:2px

    User["1. User sends command"]:::actor
    Example[">>analyze @CAGEERF :: 'cite sources'"]:::actor
    User --> Example --> Parse

    subgraph Server["MCP Server"]
        direction TB
        Parse["2. Parse operators"]:::process
        Inject["3. Inject framework + gates"]:::process
        Render["4. Render prompt"]:::process
        Decide{"6. Route verdict"}:::decision
        Parse --> Inject --> Render
    end
    Server:::server

    subgraph Client["Agent (Client)"]
        direction TB
        Execute["5. Run prompt + check gates"]:::client
    end
    Client:::clientbg

    Render -->|"Prompt with gate criteria"| Execute
    Execute -->|"Verdict + output"| Decide

    Decide -->|"PASS → render next step"| Render
    Decide -->|"FAIL → render retry prompt"| Render
    Decide -->|"Done"| Result["7. Return to user"]:::actor
```

**The feedback loop:** You send a command with operators → Server parses and injects methodology/gates → Agent executes and self-evaluates → Server routes: next step (PASS), retry (FAIL), or return result (done).

---

## Quick Start

### Agent CLI (Recommended)

**Step 1: Add the plugin marketplace** (first time only)

```bash
/plugin marketplace add minipuft/minipuft-plugins
```

**Step 2: Install the plugin**

```bash
/plugin install agent-prompts@minipuft
```

**Step 3: Try it**

```bash
>>tech_evaluation_chain library:'zod' context:'API validation'
```

<details>
<summary><strong>Why hooks matter</strong></summary>

The plugin adds hooks that fix common issues:

| Problem | Hook Fix |
|---------|----------|
| Model ignores `>>analyze` | Detects syntax, suggests correct MCP call |
| Chain step forgotten | Injects `[Chain] Step 2/5 - continue` |
| Gate review skipped | Reminds `GATE_REVIEW: PASS\|FAIL` |

Raw MCP works, but models sometimes miss the syntax. The hooks catch that. → [hooks/README.md](hooks/README.md)

</details>

**User Data**: Custom prompts stored in `~/.local/share/agent-prompts/` persist across updates.

### Generic CLI

```bash
# Install directly from GitHub
agent extensions install https://github.com/minipuft/agent-prompts-mcp

# Development Setup (Hot Reload)
# Use a symbolic link to point the extension directory directly to your source code.
# This ensures changes to hooks and prompts are reflected immediately.
rm -rf ~/.agents/extensions/agent-prompts
ln -s "$(pwd)" ~/.agents/extensions/agent-prompts
```

The extension provides:

- **MCP server** with the same tools (`prompt_engine`, `resource_manager`, `system_control`)
- **AGENTS.md** context file with usage documentation

**Optional hooks** for `>>prompt` syntax detection can be enabled manually - see `AGENTS.md` for setup instructions.

Works with the same prompts, gates, and methodologies as other agents.

### Desktop Agent

| Method | Install Time | Updates | Custom Prompts |
|--------|-------------|---------|----------------|
| **Desktop Extension** | 10 seconds | Manual | Built-in config |
| **NPX** | 30 seconds | Automatic | Via env vars |

**Desktop Extension** (one-click):

```
1. Download agent-prompts.mcpb → github.com/minipuft/agent-prompts-mcp/releases
2. Drag into Desktop Agent Settings
3. Done. Optionally set a custom prompts folder when prompted.
```

**NPX** (auto-updates):

```json
// ~/Library/Application Support/Agent/agent_desktop_config.json (macOS)
// %APPDATA%\Agent\agent_desktop_config.json (Windows)
{
  "mcpServers": {
    "agent-prompts": {
      "command": "npx",
      "args": ["-y", "agent-prompts@latest"]
    }
  }
}
```

Restart Desktop Agent. Test it:

```
>>research_chain topic:'remote team policies' purpose:'handbook update'
```

→ Returns a 4-step research workflow with methodology injection and quality gates.

### Other MCP Clients

<details>
<summary><strong>Generic MCP clients (.cursor/mcp.json, etc.)</strong></summary>

Add to your MCP config:

```json
{
  "mcpServers": {
    "agent-prompts": {
      "command": "npx",
      "args": ["-y", "agent-prompts@latest"]
    }
  }
}
```

Test: `resource_manager(resource_type:"prompt", action:"list")`

</details>

<details>
<summary><strong>Cursor 1-click install</strong></summary>

<a href="cursor://anysphere.cursor-deeplink/mcp/install?name=agent-prompts&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsImFnZW50LXByb21wdHNAbGF0ZXN0Il19">
  <img src="https://cursor.com/deeplink/mcp-install-dark.png" alt="Add to Cursor" height="28" />
</a>

</details>

<details>
<summary><strong>From Source</strong></summary>

```bash
git clone https://github.com/minipuft/agent-prompts-mcp.git
cd agent-prompts-mcp/server && npm install && npm run build
```

Then point your config to `server/dist/index.js`.

**Transport options**: `--transport=stdio` (default), `--transport=streamable-http` (recommended for HTTP).

</details>

### Custom Resources

Use your own prompts without cloning:

```json
{
  "mcpServers": {
    "agent-prompts": {
      "command": "npx",
      "args": ["-y", "agent-prompts@latest"],
      "env": {
        "MCP_RESOURCES_PATH": "/path/to/your/resources"
      }
    }
  }
}
```

Your resources directory can contain: `prompts/`, `gates/`, `methodologies/`, `styles/`.

| Override Method | Example |
|-----------------|---------|
| All resources | `MCP_RESOURCES_PATH=/path/to/resources` |
| Just prompts | `MCP_PROMPTS_PATH=/path/to/prompts` |
| CLI flag (dev) | `--prompts=/path/to/prompts` |

**Priority:** CLI flags > individual env vars > `MCP_RESOURCES_PATH` > package defaults.

See [CLI Configuration](docs/reference/mcp-tools.md#cli-configuration) for all options.

---

## What You Get

### 🔥 Hot Reload

Edit prompts, test immediately. Better yet—ask the Agent to fix them:

```text
User: "The code_review prompt is too verbose"
Agent: resource_manager(action:"update", id:"code_review", ...)
User: "Test it"
Agent: prompt_engine(command:">>code_review")  # Uses updated version instantly
```

### 🔗 Chains

Break complex tasks into steps with `-->`:

```text
analyze code --> identify issues --> propose fixes --> generate tests
```

Each step's output flows to the next. Add quality gates between steps.

### 🧠 Frameworks

Inject structured thinking patterns:

```text
@CAGEERF Review this architecture    # Context → Analysis → Goals → Execution → Evaluation → Refinement
@ReACT Debug this error              # Reason → Act → Observe loops
```

### 🛡️ Gates

Quality criteria the Agent self-checks:

```text
Summarize this :: 'under 200 words' :: 'include key statistics'
```

Failed gates can retry automatically or pause for your decision.

### ✨ Judge Selection

Let the Agent pick the right tools:

```text
%judge Help me refactor this codebase
```

The Agent analyzes available frameworks, gates, and styles, then applies the best combination.

### 📜 Version History

Every update is versioned. Compare, rollback, undo:

```text
resource_manager(action:"history", id:"code_review")
resource_manager(action:"rollback", id:"code_review", version:2, confirm:true)
```

---

## Syntax Reference

| Symbol | Name | What It Does | Example |
|:------:|:-----|:-------------|:--------|
| `>>` | Prompt | Execute template | `>>code_review` |
| `-->` | Chain | Pipe to next step | `step1 --> step2` |
| `@` | Framework | Inject methodology | `@CAGEERF` |
| `::` | Gate | Add quality criteria | `:: 'cite sources'` |
| `%` | Modifier | Toggle behavior | `%clean`, `%judge` |
| `#` | Style | Apply formatting | `#analytical` |

**Modifiers:**

- `%clean` — No framework/gate injection
- `%lean` — Gates only, skip framework
- `%guided` — Force framework injection
- `%judge` — Agent selects best resources

---

## Using Gates

```text
# Inline (quick)
Research AI :: 'use recent sources' --> Summarize :: 'be concise'

# With framework
@CAGEERF Explain React hooks :: 'include examples'

# Programmatic
prompt_engine({
  command: ">>code_review",
  gates: [{ name: "Security", criteria: ["No hardcoded secrets"] }]
})
```

| Severity | Behavior |
|----------|----------|
| Critical/High | Must pass (blocking) |
| Medium/Low | Warns, continues (advisory) |

See [Gates Guide](docs/guides/gates.md) for full schema.

---

## Configuration

Customize via `server/config.json`:

| Section | Setting | Default | Description |
|:--------|:--------|:--------|:------------|
| `prompts` | `directory` | `prompts` | Prompts directory (hot-reloaded) |
| `frameworks` | `injection.systemPrompt` | enabled | Auto-inject methodology guidance |
| `gates` | `definitionsDirectory` | `gates` | Quality gate definitions |
| `execution` | `judge` | `true` | Enable `%judge` resource selection |

---

## The Three Tools

| Tool | Purpose |
|------|---------|
| `prompt_engine` | Execute prompts with frameworks and gates |
| `resource_manager` | CRUD for prompts, gates, methodologies |
| `system_control` | Status, analytics, health checks |

```bash
prompt_engine(command:"@CAGEERF >>analysis topic:'AI safety'")
resource_manager(resource_type:"prompt", action:"list")
system_control(action:"status")
```

---

## Documentation

- **[MCP Tooling Guide](docs/reference/mcp-tools.md)** — Full command reference
- **[Prompt Authoring](docs/guides/prompt-authoring-guide.md)** — Template syntax and schema
- **[Chains](docs/guides/chains.md)** — Multi-step workflows
- **[Gates](docs/guides/gates.md)** — Quality validation
- **[Architecture](docs/architecture/overview.md)** — System internals

---

## Contributing

```bash
cd server
npm install && npm run build
npm test
npm run validate:all  # Full CI check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## License

[MIT](LICENSE)
