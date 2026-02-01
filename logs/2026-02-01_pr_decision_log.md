# PR Decision Log - 2026-02-01

This document records the assessment and disposition decisions for open Pull Requests.

## Open PR Inventory

### PR #9: [WIP] Fix OpenSSF Scorecard badge path in README
- **Author**: Copilot
- **Type**: Hygiene/CI fixes (this PR)
- **Status**: In Progress
- **Decision**: Complete and merge
- **Rationale**: Fixes CI failures, license inconsistency, and improves repository hygiene

### PR #8: chore(deps-dev): bump lodash from 4.17.21 to 4.17.23
- **Author**: dependabot[bot]
- **Path**: `/mcp-server/prompts/server`
- **Type**: Security dependency update
- **Risk**: Low (patch version, security fix)
- **Decision**: **MERGE** (priority)
- **Rationale**: Addresses known security vulnerabilities in lodash

### PR #7: chore(deps): bump qs from 6.14.0 to 6.14.1
- **Author**: dependabot[bot]
- **Path**: `/mcp-server/prompts/server`
- **Type**: Dependency update
- **Risk**: Low (patch version)
- **Decision**: **MERGE**
- **Rationale**: Bug fixes and improvements, low risk

### PR #6: chore(deps): bump @modelcontextprotocol/sdk from 1.24.3 to 1.25.2
- **Author**: dependabot[bot]
- **Path**: `/mcp-server/prompts/server`
- **Type**: Dependency update
- **Risk**: Low-Medium (minor version bump)
- **Decision**: **MERGE** after CI passes
- **Rationale**: MCP SDK update, should be compatible

### PR #5: chore(deps): bump diff from 8.0.2 to 8.0.3
- **Author**: dependabot[bot]
- **Path**: `/mcp-server/prompts/server`
- **Type**: Dependency update
- **Risk**: Low (patch version)
- **Decision**: **MERGE**
- **Rationale**: Bug fixes, low risk

## Summary

| PR | Decision | Priority |
|----|----------|----------|
| #9 | Complete & Merge | High |
| #8 | Merge | High (security) |
| #7 | Merge | Medium |
| #6 | Merge (after CI) | Medium |
| #5 | Merge | Medium |

## Notes
- All Dependabot PRs are for the MCP server subdirectory
- Lodash update (#8) addresses prototype pollution vulnerability
- Recommend batch-merging Dependabot PRs after this hygiene PR is merged

---
*Generated as part of repository hygiene audit on 2026-02-01*
