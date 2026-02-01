# HQE-Workbench Governance Model

**Version:** 1.0  
**Last Updated:** 2026-02-03

---

## 1. Overview

HQE-Workbench follows a **Benevolent Dictator Governance Model** with
evolving community input. The project is led by a single maintainer
with authority over all technical and community decisions, while
welcoming contributions from the broader open-source community.

---

## 2. Roles and Responsibilities

### 2.1 Project Lead (Benevolent Dictator)

*Current:* **Faye Hakansdotter** (@AbstergoSweden)

**Authority:**
- Final decision-making authority on all technical and governance matters
- Veto power on any proposed changes
- Control over release schedules and versioning
- Authority to appoint or remove maintainers

**Responsibilities:**
- Set project vision and roadmap
- Ensure code quality and security standards
- Manage security vulnerability disclosures
- Approve major architectural changes
- Represent the project in public forums

### 2.2 Core Contributors

**Definition:** Individuals with write access to the repository.

**Privileges:**
- Direct push access to `main` branch (discouraged; PRs preferred)
- Merge PRs after approval
- Manage issues and discussions

**Appointment:** By invitation from the Project Lead only.

### 2.3 Contributors

**Definition:** Anyone who submits a PR, issue, or discussion that is merged/accepted.

**Recognition:** Listed in git history and acknowledged in release notes.

---

## 3. Decision-Making Process

### 3.1 Technical Decisions

| Scope | Decision Maker | Input Process |
|-------|---------------|---------------|
| Bug fixes, docs | Any contributor | PR review by any maintainer |
| New features | Project Lead | RFC discussion recommended |
| API changes | Project Lead | RFC required, 7-day comment period |
| Architecture | Project Lead | RFC required, community consultation |
| Security fixes | Project Lead | Immediate action, post-hoc review |

### 3.2 Community Decisions

- Code of Conduct enforcement: Project Lead
- Banning/removal: Project Lead with 24h appeal window
- Governance changes: Project Lead approval required

---

## 4. Contribution Workflow

1. **Fork & Branch** - Create feature branch from `main`
2. **Develop** - Follow style guides in CONTRIBUTING.md
3. **Test** - Ensure CI passes locally (`npm run preflight`)
4. **Submit PR** - Fill template, reference issues
5. **Review** - At least one maintainer approval required
6. **Merge** - Squash-merge preferred, conventional commits enforced

---

## 5. Release Process

**Versioning:** Semantic Versioning (SemVer 2.0)

**Release Types:**
- **Patch (0.0.x):** Bug fixes, security patches
- **Minor (0.x.0):** New features, backwards compatible
- **Major (x.0.0):** Breaking changes (requires RFC)

**Process:**
1. Update CHANGELOG.md with release notes
2. Tag release with GPG signature: `git tag -s v0.x.0`
3. GitHub Actions builds and publishes artifacts
4. Create GitHub Release with notes

---

## 6. Security Governance

See [SECURITY.md](SECURITY.md) for full details.

- **Disclosure:** Coordinated disclosure required
- **Response Time:** 48h acknowledgment, 30-day fix target
- **Embargoes:** Critical fixes may be embargoed up to 7 days

---

## 7. Conflict Resolution

1. **Technical Disagreements:** Project Lead has final say
2. **Interpersonal Issues:** Mediation attempt, then Project Lead decision
3. **Appeals:** May be submitted privately to security contact

---

## 8. Changes to Governance

This document may only be modified by the Project Lead.
Proposed changes will be announced in a GitHub Discussion
with a 14-day comment period before taking effect.

---

## 9. Acknowledgments

This governance model is adapted from:
- Python's Benevolent Dictator For Life (BDFL) model
- Django's governance structure
- Kubernetes community principles

---

**Document History:**
- v1.0 (2026-02-03): Initial governance document
