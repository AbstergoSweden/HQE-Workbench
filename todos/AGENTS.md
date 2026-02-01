# Agent TODO Instructions

This document provides instructions for AI agents and automated tools working with the repository's TODO system.

## For AI Agents

When working on tasks in this repository:

1. **Check this directory first** before creating new TODO items elsewhere
2. **Update existing TODOs** in this directory when making progress on related work
3. **Create new TODOs here** instead of adding inline `// TODO:` comments in code
4. **Link TODOs to issues/PRs** when applicable for traceability

## TODO File Locations

| Type | Location | Description |
|------|----------|-------------|
| Current Work | `todos/current.md` | Active tasks |
| Backlog | `todos/backlog.md` | Future work, ideas |
| Completed | `todos/completed.md` | Historical record |

## Creating a TODO Entry

When you identify work that needs to be done:

1. Determine the appropriate file based on priority/timeline
2. Add an entry using the format specified in `README.md`
3. Include sufficient context for future agents/developers
4. Reference the source (issue number, user request, code location)

## Updating TODO Status

When you complete or make progress on a task:

1. Update the status field
2. Add notes about what was done
3. If completed, move to `completed.md` with completion date
4. Link to the PR/commit that addressed the task

## Do NOT

- Create scattered TODO lists in random markdown files
- Add inline TODO comments without a corresponding entry here
- Delete historical TODO entries (preserve for context)
- Ignore existing TODOs when working on related areas
