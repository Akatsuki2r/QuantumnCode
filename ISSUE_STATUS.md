# Issue Tracking Status - 2026-04-28

> This document tracks the critical problems, technical debt, and major blockers identified in the QuantumnCode Rust repository.
> **Note**: The original tracking document (`QUANTUM_TODO.md`) was written for a TypeScript version of the project. This is a Rust implementation that differs significantly.

---

## Critical Issues (Blocking)

### 1. Router module is not connected to the main QueryEngine

**Status**: PARTIALLY RESOLVED

The router IS connected in the Rust codebase:
- `app.rs:280-347` - `route_prompt()` method routes user prompts
- `agent/executor.rs:71` - Agent loop uses `route()` for routing decisions
- `router/mod.rs` - Main routing function with 7 layers

**Remaining Gap**: Tool policy enforcement in the tool execution loop
- `pick_tools()` returns a `ToolPolicy` with allowed/disallowed tools
- But the agent executor doesn't check `is_tool_allowed()` before executing tools
- Router decisions are logged but not enforced on tool calls

---

### 2. Rust module tests incomplete

**Status**: RESOLVED

The original issue mentioned `file_ops.rs` and `token_estimate.rs` - these files do NOT exist in the Rust codebase.

Actual test coverage:
- `src/router/tests.rs` - 155 tests covering all 7 router layers
- `src/router/analyzer.rs` - inline tests for intent classification
- `src/router/mode.rs` - inline tests for mode selection
- `src/router/memory.rs` - inline tests for memory policy
- `src/router/context.rs` - inline tests for budget allocation
- `src/router/model.rs` - inline tests for model tier selection

**Test Results**: All 155 tests pass
```
cargo test 2>&1 | tail -5
test result: ok. 155 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

### 3. Performance regressions not detected (missing CI benchmarks)

**Status**: NOT RESOLVED

- `Cargo.toml` has `criterion = "0.5"` as dev-dependency
- No `benches/` directory exists
- No benchmark suite configured
- No CI pipeline for performance regression detection

**Action Needed**: Create `benches/router_bench.rs` with Criterion benchmarks

---

## Major Issues

### 4. Memory strategy not enforced in live query pipeline

**Status**: PARTIALLY RESOLVED

Memory policy IS selected in routing:
- `router/memory.rs:8-36` - `pick_memory_policy()` returns policy
- `router/types.rs` defines `MemoryPolicy` enum (None/Recent/Relevant/Full)

**Remaining Gap**: Policy is NOT used to load memory
- `agent/executor.rs` does not call any memory loading function
- `app.rs` does not filter context by relevance
- Memory entries are loaded blindly without relevance filtering

---

### 5. Token/compression limits trimming logic not active

**Status**: NOT RESOLVED

- `pick_budget()` returns `ContextBudget` correctly
- `agent_token_budget()` calculates available tokens
- NO `enforce()` function exists anywhere
- NO trim callbacks are registered
- Context budget is calculated but not applied to message history

---

### 6. Mode implementation defined but not enforced

**Status**: PARTIALLY RESOLVED

Mode selection IS working:
- `router/mode.rs` - `pick_mode()` returns correct mode
- `router/types.rs:100-157` - `AgentMode` enum (Chat/Plan/Build/Review/Debug)
- Mode is included in routing decision

**Remaining Gap**: Mode restrictions not enforced
- Agent executor doesn't check if mode allows tool execution
- Plan mode should be read-only but can still call Write/Edit tools
- Build mode enables all tools regardless of intent

---

### 7. JS fallback for Rust core is incomplete

**Status**: NOT APPLICABLE

The Rust project does NOT use a JS fallback architecture:
- This is a pure Rust CLI application
- No NAPI-RS bindings or TypeScript fallback layer
- `npm/` directory contains only wrapper scripts for npm distribution
- JS wrapper shells out to the Rust binary

---

## Moderate Issues

### 8. No integration/unit tests for core CLI tools

**Status**: NOT RESOLVED

CLI tools exist at `src/tools/`:
- `read_file.rs`, `write_file.rs`, `bash.rs`, `grep.rs`, `glob.rs`

**NO tests exist** for any of these tools:
```bash
$ grep -r "#\[test\]" src/tools/
# (no test modules in tools directory)
```

---

### 9. Config parsing lacks edge case handling/tests

**Status**: UNKNOWN

- `src/config/settings.rs` exists with Settings struct
- No test file found for config module
- Edge case handling unclear (missing keys, invalid values, etc.)

---

### 10. Session save/resume not tested for consistency

**Status**: NOT IMPLEMENTED

From `src/commands/session.rs`:
```rust
// TODO: Load session and start interactive mode
println!("Session resumption will be implemented in Phase 3.");
// ...
// TODO: Save current session state
println!("Session saving will be implemented in Phase 3.");
```

`delete_session()` IS implemented, but save/resume are stubs.

---

### 11. Token budget tracking not wired to real LLM calls

**Status**: PARTIALLY RESOLVED

Token estimation exists:
- `app.rs:422-426` - `estimate_tokens()` function
- `context.rs:31-33` - `agent_token_budget()` calculation

**Remaining Gap**: Not enforced on actual API calls
- No check before sending messages to provider
- No tracking of actual token usage from API responses
- Budget calculated but not used to trim conversation

---

### 12. Unused/dead dependencies

**Status**: UNKNOWN

Dependencies appear reasonable for a Rust CLI project with TUI.
No obvious dead dependencies. Full audit not performed.

---

## Minor Issues

### 13. No E2E tests for routing, mode switching, provider selection

**Status**: NOT RESOLVED

Integration test coverage:
- Router unit tests exist (155 tests)
- NO integration tests across components
- NO E2E tests for complete user workflows

---

### 14. Documentation missing for configuration reference and performance tuning

**Status**: PARTIALLY RESOLVED

`README.md` exists with:
- Installation instructions
- Usage examples
- Configuration file format
- Provider setup

**Missing**:
- Complete configuration reference (all keys, types, defaults)
- Performance tuning guide
- Architecture documentation

---

### 15. Directory structure includes placeholders/empty modules

**Status**: RESOLVED

All directories have meaningful implementations:
- `src/router/` - 7-layer routing engine (12 files)
- `src/agent/` - Tool execution loop
- `src/providers/` - 4 provider implementations
- `src/tools/` - 5 CLI tools
- `src/commands/` - 10 command modules

No placeholder or empty modules found.

---

## Summary Table

| Issue | Severity | Status |
|-------|----------|--------|
| Router not connected | Critical | Partial - Connected but not enforced |
| Rust tests incomplete | Critical | Resolved - 155 tests pass |
| No CI benchmarks | Critical | Not Resolved |
| Memory not enforced | Major | Partial - Selected but not used |
| Token trimming not wired | Major | Not Resolved |
| Mode not enforced | Major | Partial - Selected but not enforced |
| JS fallback incomplete | Major | N/A - Pure Rust project |
| CLI tool tests missing | Moderate | Not Resolved |
| Config edge cases | Moderate | Unknown |
| Session save/resume | Moderate | Not Implemented |
| Token budget not wired | Moderate | Partial |
| Dead dependencies | Moderate | Unknown |
| E2E tests missing | Minor | Not Resolved |
| Docs missing | Minor | Partial |
| Empty modules | Minor | Resolved |

---

## Priority Actions

1. **Wire tool policy enforcement** - Check `is_tool_allowed()` before tool execution
2. **Implement session save/resume** - Already stubbed in `session.rs`
3. **Add CLI tool tests** - Create `src/tools/tests.rs`
4. **Create benchmark suite** - Add `benches/router_bench.rs`
5. **Wire token budget enforcement** - Apply context budget to message history
6. **Add memory policy enforcement** - Load memory based on policy

---

*Generated: 2026-04-28*
*Based on codebase audit of `/home/nahanomark/Documents/QuantumCode`*