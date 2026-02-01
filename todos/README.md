# TODOs Directory

This directory is the **single source of truth** for all past and current TODO items in the repository.

## Purpose

- Centralize TODO tracking instead of scattering items across the codebase
- Maintain historical context for completed and abandoned tasks
- Provide clear ownership and prioritization for pending work

## Structure

- **Current TODOs**: Active tasks that need attention
- **Completed TODOs**: Historical record of finished work (for context)
- **Archived TODOs**: Tasks that were abandoned or superseded

## Guidelines

1. **Do NOT** add inline `// TODO:` comments in code without a corresponding entry here
2. **Do** include context, rationale, and acceptance criteria for each TODO
3. **Do** update status when work begins, progresses, or completes
4. **Do** preserve history - do not delete completed items, mark them done

## File Format

TODO files should use markdown with the following template:

```markdown
## TODO: [Brief Title]

- **Status**: [Open | In Progress | Done | Archived]
- **Priority**: [Critical | High | Medium | Low]
- **Owner**: [@username or Team Name]
- **Created**: YYYY-MM-DD
- **Updated**: YYYY-MM-DD

### Description
[Detailed description of the task]

### Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

### Notes
[Any additional context, links, or references]
```

## See Also

- [AGENTS.md](./AGENTS.md) - Instructions for AI agents and automation
