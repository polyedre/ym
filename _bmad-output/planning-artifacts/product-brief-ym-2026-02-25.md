---
stepsCompleted: [1, 2, 3, 4, 5]
inputDocuments: ["brainstorming-ym-2026-02-25.md"]
date: 2026-02-25
author: Polyedre
---

# Product Brief: ym

## Executive Summary

**ym** is a focused, Unix-philosophy CLI tool for searching and patching YAML files. It addresses a critical gap in the DevOps toolchain: engineers managing Kubernetes configs and Hiera hierarchies need to search YAML by full key path (not just grep), and patch multiple values programmatically—neither `yq` nor `jq` fill this need. ym is the structural search + patch tool that understands YAML hierarchy, enabling engineers to automate config rollouts, prepare PRs, and find specific keys without guessing.

---

## Core Vision

### Problem Statement

Engineers managing configuration files (Kubernetes, Hiera, etc.) spend significant time searching YAML files for specific keys. Traditional `grep` fails because it matches partial strings—searching for "username" returns hundreds of hits without context, forcing manual inspection to find `nova_settings.db.username` vs `barbican_settings.username`. Current tools like `yq` require knowing the exact path upfront; they're query tools, not search tools.

The secondary pain point: bulk configuration updates (bump versions, increase replica counts across resources) require custom scripting or complex tool chains. Engineers need a way to patch YAML values programmatically as part of automation pipelines.

### Problem Impact

- **Search friction**: Manual grep + inspection wastes time and introduces errors
- **Automation gap**: No tool bridges "I need to find patterns" and "I need to patch values"
- **Pipeline blocker**: Configuration rollouts and PR generation require custom scripts
- **No structural understanding**: Generic text tools don't understand YAML hierarchy

### Why Existing Solutions Fall Short

**`grep`**: Only matches text, no structural awareness. Returns all occurrences without context.

**`yq`**: Requires exact path knowledge upfront (e.g., `.nova_settings.db.username`). It's a query tool, not a search tool. No regex-based key discovery.

**`jq`**: Similar to `yq`—excellent for precise queries, but not for "find all keys matching this pattern."

**Custom scripts**: Each use case requires Python/Ruby/etc., adding complexity and reducing reusability.

### Proposed Solution

**ym** provides three core operations:

1. **grep** – Search YAML files by key pattern (regex) across one or many files. Returns `file:full.key.path=value` format. Enables discovery without prior knowledge of exact paths.

2. **set** – Update YAML values at specified key paths. Supports multiple updates in single invocation. Direct path specification (no regex), clean and predictable.

3. **unset** – Remove keys from YAML files. Direct path specification.

**Design philosophy**: Do one thing well. Understand YAML structure. Compose with Unix pipes. No batch stdin, no assumptions—keep it simple and focused.

**Use cases enabled**:
- Find all `.*_settings\..*\.password` keys across a config directory
- Bump version numbers programmatically in deployment manifests
- Increase replica counts across multiple Kubernetes configs
- Generate PRs with automated config updates
- Validate config patterns in CI/CD pipelines

### Key Differentiators

1. **Structural Search**: Unlike `grep`, ym understands YAML hierarchy. It matches against the full dotted key path (`nova_settings.db.username`), not just text fragments.

2. **Fills the Gap**: `yq` requires exact paths; `ym` searches. They don't overlap—ym solves the "discovery" problem, `yq` solves the "I know what I want" problem.

3. **Unix Philosophy**: Simple, focused operations. Works with pipes. Clean output format. Designed for scripting and automation, not interactive use.

4. **Generic Foundation**: Works with any YAML—Kubernetes, Hiera, application configs. Abstractions can be built on top. No opinionation, no assumptions about use case.

5. **DevOps-First**: Built for the actual workflows engineers use: searching configs, patching infrastructure, automating rollouts.

---

## Target Users

### Primary Users: GitOps & Config Management Engineers

**Who they are:**
Developers and operators managing large YAML configuration repositories—Kubernetes manifests, Hiera hierarchies, application configs—across multiple environments (prod, preprod, dev). They own the "source of truth" for infrastructure and application configuration.

**Their world:**
- Managing hundreds or huge YAML files across multiple repos and environments
- Frequently need to understand what a specific configuration value is set to in different places
- Spend time manually grepping through configs, then inspecting results to find the right key
- Expected to understand config drift and consistency across environments
- Make configuration changes as part of code reviews, debugging, and infrastructure updates

**The problem they face:**
Grepping for a key returns noise—all partial matches, no structural context. To find `nova_settings.db.username` across a repo, they grep "username", get 200 results, and manually inspect each one. It's tedious, error-prone, and blocks their understanding.

**Their success with ym:**
One command: `ym grep .*_settings\..*\.password` gives them exactly what they need—the full key path and value across all their configs. They understand their configuration landscape without guessing.

**Adoption path:**
1. **Manual discovery phase:** Use `ym grep` to understand configs faster than grep. "This is better." If they see value, they keep using it.
2. **Automation phase:** Integrate into scripts—validate consistency, automate config updates, prepare PRs, run CI/CD checks. Natural progression as they find their own use cases.
3. **Long-term:** Becomes part of their config management toolkit. Easy enough to install, useful enough to keep.

**Barrier to adoption:**
The problem must be painful enough to justify installing a new tool. They have large YAML configuration challenges. If ym solves their immediate problem better than grep, they adopt.

### Secondary Users

Automation systems, validation frameworks, and scripts that need to work with YAML configurations. As the primary user base grows and integrates ym into workflows, secondary tools and systems will compose on top of it.

### User Journey

**Discovery & Installation:**
User discovers `ym` while searching for a better way to grep YAML configs. Installation is simple (brew, pip, apt, etc.)—low friction.

**First Use:**
They run a familiar grep pattern adapted to ym: `ym grep .*_settings\..*\.password`. The output is clean, structured, and immediately useful. "This is what I needed."

**Understanding Value:**
Using ym for a few manual searches—understanding what a value is across environments, checking for consistency, spotting anomalies. Saves time. Works better than grep.

**Integration:**
They think: "I could use this in a script." They start composing ym commands into automation—config validation, update preparation, CI/CD checks. They find their own use cases.

**Long-term:**
ym becomes a standard part of how they work with YAML. Part of their toolkit, not a one-off experiment.

---

## Success Metrics

### User Success

Users succeed with ym when they achieve faster, more accurate YAML searching than grep, and integrate it into their regular workflows.

**User Success Indicators:**

- **Search Quality:** Users get clean, structured results with full key paths—no noise, no false matches. One search gives them the answer they need.
- **Performance:** Tool runs at grep-level speed or better. No slowdown vs. existing workflows.
- **Adoption in Workflow:** Users reach for ym instead of grep for key discovery. Moved from "I tried it" to "this is my standard tool."
- **Integration into Automation:** Users compose ym into scripts and validation tools. They find their own use cases beyond manual search.
- **Team Sharing:** Users recommend ym to teammates. Word-of-mouth adoption.

### Business Objectives & Growth

**Philosophy:** Open source, organic growth, solves the problem first. No sales, no acquisition targets. Success is measured by genuine usefulness and community adoption.

**Business Success at 6-12 Months:**

- **Organic Adoption:** Active user base within DevOps, GitOps, and config management communities. Users discovering ym through word-of-mouth, blog posts, discussions.
- **Community Health:** Active GitHub presence—consistent usage signals, community contributions, issues that reflect real-world usage patterns.
- **Integration Success:** Evidence that users are composing ym into scripts, validation pipelines, and automation workflows.
- **Reliability:** Tool performs reliably without regressions or performance issues.

### Key Performance Indicators

- **Adoption Indicator:** GitHub stars and active clones indicate community interest and viral potential
- **Usage Signal:** Open issues, pull requests, and discussion activity show real usage and engagement
- **Quality Metric:** Tool maintains grep-level performance (search time comparable to standard grep on same datasets)
- **Problem Solved:** Users solving YAML search problems faster and more accurately than grep—measured through testimonials, issue discussions, and community feedback
- **Community Momentum:** Regular contributors, forks, and downstream projects built on ym

Strategic Alignment

Success for ym is **genuine usefulness in solving a real problem** that existing tools (grep, yq, jq) don't solve well. The metrics reflect organic adoption, reliability, and the ability to become a standard tool in the DevOps/GitOps toolkit—not through marketing, but through solving the problem better than alternatives.

---

## MVP Scope

### Core Features

**ym** MVP delivers three essential operations for YAML manipulation:

#### 1. grep – Search YAML by Key Path
- Search single YAML files by exact key path match
- Output format: `file:full.key.path=value` (one result per line)
- Returns all matching keys with their values
- Clean, parseable output suitable for piping and scripting
- Example: `ym grep nova_settings.db.password config.yaml`

#### 2. set – Update YAML Values
- Set values at specified key paths (exact path, no patterns)
- Support multiple key-value pairs in single command
- Creates nested structure if path doesn't exist
- Example: `ym set config.yaml nova_settings.username=toto barbican_settings.username=myuser`

#### 3. unset – Delete Keys
- Remove keys at specified paths
- Exact path matching (no patterns)
- Clean removal without leaving empty structures
- Example: `ym unset config.yaml nova_settings.password`

#### Implementation Details

- **Single file operations**: Process one file per invocation (focus, composability)
- **Simple YAML structures**: Support basic mappings and nested keys (no special array handling in MVP)
- **No dependencies**: Pure Rust implementation, zero external dependencies
- **Linux target**: Build for Linux; cross-platform support deferred
- **Performance**: Operations complete at grep-level speed or faster

### Out of Scope for MVP

Explicitly deferred to maintain focus and deliver quickly:

- **Regex patterns**: Exact key matching only (regex search deferred to v2)
- **Recursive directory traversal** (`-R` flag): Single-file focus for MVP
- **Batch stdin operations**: One operation per invocation
- **Multiple output formats**: Standard text format only (JSON, YAML exports future)
- **Interactive mode**: Command-line only, no interactive shell
- **Config files or options**: Simple CLI args only
- **Array-specific handling**: Arrays treated as nested structures; complex array operations deferred
- **Cross-platform**: Linux focus for MVP (macOS, Windows future)

### MVP Success Criteria

The MVP is successful when:

- **Functional**: All three operations (grep, set, unset) work end-to-end on real YAML files
- **Output Quality**: grep produces clean, parseable `file:key.path=value` format
- **Reliable**: No data loss, no corruption of YAML structure
- **Performance**: Operations execute at standard grep speed or better
- **Usable**: Commands are intuitive and work as users expect
- **Standalone**: Builds and runs with zero dependencies on Linux

### Future Vision

With proven MVP success, future enhancements build on this foundation:

- **Pattern Matching**: Regex support for key discovery (version 2)
- **Bulk Operations**: Recursive directory traversal, batch stdin operations
- **Advanced Features**: Array element manipulation, conditional updates, validation
- **Ecosystem**: Multiple output formats (JSON, YAML), config file support, plugin architecture
- **Platform Expansion**: Cross-platform support (macOS, Windows), distribution channels (brew, cargo, apt)
- **Intelligence**: Transformation pipelines, schema validation, conflict detection

The MVP establishes ym as a reliable, focused tool. Future versions expand capability while maintaining the Unix philosophy: do one thing well, compose with pipes, integrate seamlessly.
