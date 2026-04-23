# Quantum Code Implementation TODO

## Status Legend

- [ ] Not Started
- [/] In Progress
- [x] Completed
- [!] Blocked

> **Last Audit**: 2026-04-23 — Corrected Phase 5.1 status (router NOT connected to TUI)

---

## Phase 1: Core Router Foundation — ✅ COMPLETE

### 1.1 Router Architecture Design

- [x] Design router module structure → `src/router/` (12 files, ~148KB)
- [x] Define TypeScript interfaces → `types.ts` (404 lines: `TaskAnalysisInput`, `TaskAnalysisResult`, `RoutingDecision`)
- [x] Define routing decision types → `types.ts:238-267` (`RoutingDecision` with 15 fields)
- [x] Create router configuration schema → `types.ts:330-404` (`RouterConfig` + `DEFAULT_ROUTER_CONFIG`)

### 1.2 Intent Classification Layer

- [x] Implement task intent classifier → `TaskAnalyzer.ts:29-165` (16 intents via regex pattern matching)
  - Intents: chat, explain, summarize, plan, analyze, review, implement, refactor, fix, test, search, inspect, execute, debug, configure, unknown
- [x] Define intent categories → `types.ts:47-71` (5 categories: conversational, analytical, implementation, operational, configuration)
- [x] Create signal extraction → `TaskAnalyzer.ts:167-222` (complexity keywords, tool patterns, file references)
- [x] Implement complexity estimation → `TaskAnalyzer.ts:335-491` (weighted point system: 5 levels from trivial to heavy)

### 1.3 Mode Management

- [x] Implement mode state machine → `ModeManager.ts:71-315` (5 modes: chat, plan, build, review, debug)
  - Valid transitions defined per mode with context preservation rules
- [x] Create mode-specific prompt builders → `ModeManager.ts:71-315` (`promptMods` per mode with mode-specific instructions)
- [x] Implement mode switching logic → `ModeManager.ts:413-449` (validate transition + `preserveContextForTransition()`)
- [x] Add mode persistence → `ModeManagerWithPersistence` (debounced 500ms disk writes to session directory)

---

## Phase 2: Model & Tool Selection — ✅ COMPLETE

### 2.1 Model Tier Selection

- [x] Define model tier interfaces → `types.ts:144-159`, `ModelSelector.ts:22-46` (4 tiers: local, fast, standard, capable)
- [x] Implement complexity → tier mapping → `ModelSelector.ts:325-339` (switch on complexity level)
- [x] Create escalation/de-escalation logic → `ModelSelector.ts:345-366` (`upgradeTier()`, `downgradeTier()`, escalation triggers)
- [x] Add cost-aware selection → `ModelSelector.ts:293-299` (`estimateCost()` with budget check + downgrade)

### 2.2 Tool Policy Engine

- [x] Implement tool necessity estimation → `TaskAnalyzer.ts:497-536` (pattern matching + intent-based defaults)
- [x] Create tool filtering by task type → `ToolPolicyManager.ts:180-268` (per-intent allowed/disallowed/approval lists)
- [x] Implement batch tool planning → `ParallelToolExecutor.ts:477-522` (`groupIntoBatches()` with safety classification)
- [x] Add tool sparseness optimization → `ToolPolicyManager.ts:288-325` (activation levels: minimal/standard/full per mode)

### 2.3 Context Strategy

- [x] Implement context relevance scoring → `ContextStrategy.ts:362-378` (keyword matching against context entries)
- [x] Create context compression → `ContextStrategy.ts:263-294` (85% warn threshold, 100% force-trim at 50% target)
- [x] Design working vs persistent memory → `MemoryStrategy.ts:19-23` (4 types: project, user, session, working)
- [x] Implement context budget management → `ContextStrategy.ts:23-52` (4 budgets: minimal=4K, relevant=16K, standard=50K, comprehensive=100K)

### 2.4 Router Test Suite

- [x] Create comprehensive router test file → `src/router/__tests__/`

---

## Phase 3: Execution Optimization — ⚠️ PARTIAL (4/12 done)

### 3.1 Latency Optimization

- [x] Implement parallel tool execution → `ParallelToolExecutor.ts` (ExecutionGraph DAG, safe/isolated/sequential classifications)
- [x] Identify Rust migration hot paths
  - **Candidates**: `analyzePrompt()`, `estimateTokens()`, `globSearch()`, `grepSearch()` — all already ported to Rust
- [ ] Create predictive tool pre-loading
  - **Implementation idea**: After routing, if intent is `implement` and `fileScope > 3`, pre-read the identified files before the LLM responds
  - **Where**: Should hook into the query pipeline between `route()` and `callLLM()`
- [ ] Add response caching for repeated queries
  - **Implementation idea**: LRU cache keyed on `hash(systemPrompt + prompt)`, invalidated on file changes
  - **Where**: Wrapper around `callLLM()` in query pipeline

### 3.2 Token Optimization

- [x] Add token budget tracking → `TokenBudgetTracker.ts` (496 lines, singleton `TokenBudgetManager`, per-session tracking)
- [ ] Implement aggressive context trimming
  - **What exists**: `ContextStrategy.ts` has compression triggers at 85%/100%, `trimCallbacks` array in `TokenBudgetTracker`
  - **What's missing**: No trim callbacks are actually registered. Need to wire `enforce()` into the query loop
  - **Implementation**: Register callbacks that: (1) summarize old conversation turns, (2) drop low-relevance memory entries, (3) truncate large tool results
- [ ] Create summary injection points
  - **What's needed**: After every N turns (configurable, default 5), summarize conversation history and replace old turns with a compressed summary
  - **Where**: Between turns in the query loop, triggered by `TokenBudgetTracker.isApproachingLimit()`
- [ ] Design minimal prompt assembly for simple tasks
  - **What's needed**: For `trivial` complexity tasks, skip: skill manifest, memory loading, git context. Only inject: mode instruction + tool schemas
  - **Savings**: ~400 tokens per trivial query (from 600 → 200 baseline)

### 3.3 Memory Strategy

- [ ] Implement relevance-based memory loading
  - **What exists**: `MemoryStrategy.ts` has `calculateRelevance()` (keyword, tag, file reference, recency scoring) and `filterByRelevance()`
  - **What's missing**: Not wired to the query loop. `getUserContext()` in `context.ts` loads all CLAUDE.md content blindly
  - **Implementation**: Replace blind CLAUDE.md loading with `memoryStrategyManager.filterByRelevance(entries, policy, budget)`
- [ ] Create memory write-back policies
  - **What exists**: `MemoryStrategy.ts:shouldPersist()` (relevance thresholds per memory type)
  - **What's missing**: No caller invokes this. Need post-turn hook to extract and persist key findings
- [ ] Design session vs project memory separation
  - **What exists**: `MemoryType` enum with `project` | `user` | `session` | `working`
  - **What's missing**: Types are defined but not enforced. All memory is loaded the same way via `getMemoryFiles()`
- [ ] Add memory expiration and cleanup
  - **Not implemented at all**: Need: TTL per memory type (session: end of session, working: end of task, project: persistent)

---

## Phase 4: Rust Integration Layer — ✅ CORE DONE, Testing Missing

### 4.1 Rust Foundation

- [x] Set up Cargo workspace → `rust/Cargo.toml` (`quantum-code-core`, cdylib + rlib, NAPI-RS 2.16)
- [x] Create NAPI bindings → `rust/src/lib.rs` (4 NAPI exports: `glob_search`, `grep_search`, `estimate_tokens`, `analyze_prompt`)
- [x] Design TS/Rust interface → `src/rust/bindings.ts` (376 lines, lazy module loading, JS fallbacks for every function)

### 4.2 Performance-Critical Rust Modules

- [x] Port file indexing (glob) → `rust/src/file_ops.rs` (WalkDir + glob crate, returns `Vec<String>`)
- [x] Port content search (grep) → `rust/src/file_ops.rs` (regex crate, returns `Vec<GrepResult>`)
- [x] Port router hot paths → `rust/src/router.rs` (342 lines, `lazy_static!` + `RegexSet`, `PromptAnalysis` struct)
- [x] Port token estimation → `rust/src/token_estimate.rs` (char/4 + symbol/newline adjustments)

### 4.3 Integration Testing

- [ ] Create Rust module test suite
  - **Current state**: Only `rust/src/router.rs` has 5 tests. `file_ops.rs` and `token_estimate.rs` have zero tests
  - **Needed**: Unit tests for all functions, edge cases (empty input, unicode, binary content, paths with spaces)
- [ ] Implement benchmarking framework
  - **Current state**: `Cargo.toml` has `criterion = "0.5"` as dev-dependency and `[[bench]] name = "router_bench"` configured
  - **Needed**: Create `rust/benches/router_bench.rs` with benchmarks for `analyze_prompt`, `estimate_tokens`, `glob_search`, `grep_search`
- [ ] Add performance regression tests
  - **Needed**: CI step that runs benchmarks and fails if regression > 10%

---

## Phase 5: Mode Implementation — ⚠️ PARTIAL (Router exists, NOT integrated)

> **BLOCKER**: The router module (`src/router/`) is fully implemented but NOT connected to
> the actual TUI query loop that sends prompts to the LLM. The `route()` is only called
> in `app.rs:route_prompt()` for dropdown model selection — NOT in the message sending flow.

### 5.1 Router ↔ QueryEngine Integration — ❌ NOT CONNECTED

- [ ] Call `router.route(prompt, cwd)` before each LLM API call in `src/tui/event.rs`
  - **Current state**: `event.rs` send_message() does NOT call route()
  - **Where it should go**: Before calling `app.send_message_to_provider()`
- [ ] Inject `ModeManager.getPromptModifications()` into system prompt assembly
  - **Current state**: Mode instructions exist but not injected into prompts
- [ ] Gate tool availability via `ToolPolicyManager.isToolAllowed()` in the tool-call loop
  - **Current state**: All tools passed to LLM, no filtering
- [ ] Apply `ContextStrategy.allocateBudget()` to limit conversation history before sending
  - **Current state**: No context budget enforcement in message sending
- [ ] Call `TokenBudgetTracker.updateUsage()` after each turn with actual token counts
  - **Current state**: Not implemented
- [ ] Wire `ModeManager.transitionTo()` as a user-invocable command (e.g., `/mode plan`)
  - **Current state**: Slash commands exist but mode switching not wired to router

### 5.2 Plan Mode

- [ ] Implement plan-only execution path (read-only tools only)
- [ ] Create structured plan output format (Analysis → Approach → Steps → Risks → Dependencies)
- [ ] Add dependency and risk identification (extract from `TaskAnalyzer.analyze()` signals)
- [ ] Implement plan → build transition (user confirms plan, mode auto-switches)

### 5.3 Build Mode

- [ ] Implement execution-focused prompt assembly (include implementation context)
- [ ] Create progress checkpoint reporting (after each tool call, report: files changed, tests run)
- [ ] Add incremental execution support (resume from last checkpoint)
- [ ] Implement build → review transition (auto-trigger on commit)

### 5.4 Chat Mode — ✅ COMPLETE

- [x] Implement lightweight conversation path (minimal context, no file loading)
- [x] Create minimal tool activation (`ToolPolicyManager.ACTIVATION_MINIMAL`)
- [x] Add fast response optimization (skip memory loading, minimal system prompt)
- [x] Implement chat → plan escalation (detect complexity > simple → suggest mode switch)

---

## Phase 6: Quality & Testing — ❌ MOSTLY NOT STARTED

### 6.1 Unit Tests

- [x] Router decision tests → `src/router/__tests__/`
- [ ] Mode transition tests (validate: legal transitions succeed, illegal transitions throw)
- [ ] Tool policy tests (validate: per-intent policies match expected allowed/disallowed)
- [ ] Context strategy tests (validate: budget allocation per complexity level)
- [ ] Memory strategy tests (validate: relevance filtering, shouldPersist thresholds)
- [ ] Token budget tests (validate: enforcement at 85%/100%, auto-trim behavior)

### 6.2 Integration Tests

- [ ] End-to-end routing tests (prompt → route → verify: intent, complexity, mode, tier, tools, budget)
- [ ] Mode switching tests (simulate multi-turn conversation with mode changes)
- [ ] Memory persistence tests (write → restart → read back)
- [ ] Rust parity tests (verify JS fallback matches Rust output for all functions)
- [ ] Performance benchmarks (routing < 1ms, startup < 100ms, token budget < 600)

### 6.3 Documentation

- [x] Router architecture docs → `research_01_architecture_and_build.md`
- [x] Router deep dive → `research_02_router_deep_dive.md`
- [x] Skills/agents/optimization → `research_03_skills_agents_optimization.md`
- [x] TODO audit & roadmap → `research_04_todo_audit_and_roadmap.md`
- [ ] Configuration reference (all config keys, defaults, overrides)
- [ ] Performance tuning guide (model selection, context budgets, quantization)

---

## Phase 7: Quantum Code Port — ❌ NOT STARTED

> This is the new work: porting the existing codebase to a minimalistic, optimized version.
> See `research_04_todo_audit_and_roadmap.md` for full implementation blueprint.

### 7.1 Minimal Query Pipeline

- [ ] Create `query.ts` (< 2K lines — async generator, streaming, tool-call loop)
- [ ] Implement LLM provider abstraction (`AnthropicProvider`, `OllamaProvider`, `LlamaCppProvider`)
- [ ] Wire router into query pipeline (`route()` → system prompt + tool filter + budget)
- [ ] Support both API and local model backends

### 7.2 Core Tools (8 tools, replacing 40)

- [ ] `read` — FileRead (path, optional line range)
- [ ] `edit` — FileEdit (path, old text, new text)
- [ ] `write` — FileWrite (path, content)
- [ ] `bash` — Shell command execution
- [ ] `glob` — File pattern search
- [ ] `grep` — Content search
- [ ] `search` — Web search
- [ ] `ask` — Ask user for clarification

### 7.3 Optimized System Prompts (target: 600 tokens total)

- [ ] Base identity prompt (50 tokens: role + cwd + available tools)
- [ ] Mode instructions (30 tokens each: structured output markers)
- [ ] Compressed tool schemas (120 tokens for 8 tools: name + params + 1-line desc)
- [ ] Dynamic skill manifest (100 tokens: name + desc only, content lazy-loaded)
- [ ] Trimmed git context (100 tokens: branch + status + 3 recent commits)

### 7.4 Minimal Skill System

- [ ] Skill loader (< 100 lines, single directory, SKILL.md format)
- [ ] 5 bundled skills: commit, review, debug, test, plan
- [ ] Frontmatter-only system prompt injection (lazy content loading on invoke)

### 7.5 Local Model Support

- [ ] `llama.cpp` server integration (OpenAI-compatible HTTP API)
- [ ] Ollama integration (HTTP API)
- [ ] KV cache persistence for multi-turn sessions
- [ ] Quantization-aware prompt formatting (structured markers, common words)
- [ ] GPU dispatch (CUDA/Vulkan) with automatic fallback to CPU

### 7.6 CLI Interface (No React/Ink)

- [ ] Minimal REPL using `readline` or direct stdin
- [ ] ANSI-colored output (picocolors, 3KB vs chalk's 30KB)
- [ ] Streaming output with live typing effect
- [ ] Slash commands: `/mode`, `/cost`, `/skills`, `/config`, `/compact`

### 7.7 Dependency Reduction (74 → ~12 runtime deps)

- [ ] Replace Commander.js with `cac` (5KB) or custom arg parser
- [ ] Replace lodash-es with native ES methods
- [ ] Replace chalk with picocolors
- [ ] Drop: GrowthBook, Sentry, OpenTelemetry, React, Ink
- [ ] Keep: @anthropic-ai/sdk, zod, better-sqlite3, ignore

---

## Current Sprint

### Completed Tasks (Phases 1-2, 4.1-4.2)

1. [x] Full router module: 12 files, 7 manager classes, all integrated
2. [x] Intent classification: 16 intents, regex-based, < 1ms execution
3. [x] Complexity estimation: keyword-weighted scoring, 5 levels
4. [x] Mode state machine: 5 modes, validated transitions, context preservation
5. [x] Model tier selection: 4 tiers, escalation/de-escalation, cost-aware
6. [x] Tool policy engine: per-intent tool policies, safety classifications, activation levels
7. [x] Context strategy: 4 budget tiers, compression triggers
8. [x] Memory strategy: 4 types, relevance scoring, persistence decisions
9. [x] Token budget tracking: per-session tracking, auto-trim callbacks
10. [x] Parallel tool execution: ExecutionGraph DAG, safety-based parallelism
11. [x] Rust native module: 4 NAPI exports (glob, grep, tokens, prompt analysis)
12. [x] Research documentation: 4 comprehensive research documents
13. [x] **CORRECTED** Phase 5.1 status: Router is NOT connected to TUI query loop

### Next Up (Priority Order)

1. **Router ↔ TUI integration** (Phase 5.1) — CRITICAL BLOCKER
   - `app.rs:route_prompt()` only used for dropdown model selection, NOT in message flow
   - Need to call `route()` in `send_message_to_provider()` before each LLM call
   - Wire mode instructions, tool filtering, and context budget into message pipeline
2. **Context trimming** (Phase 3.2) — Wire `enforce()` + register trim callbacks
3. **Memory loading** (Phase 3.3) — Replace blind CLAUDE.md with `filterByRelevance()`
4. **Rust benchmarks** (Phase 4.3) — Create Criterion benchmarks, validate speedup
5. **Mode tests** (Phase 6.1) — Validate transitions, policies, budgets

---

## Metrics to Track

### Performance Metrics

| Metric               | Target  | Current  | How to Measure                               |
| -------------------- | ------- | -------- | -------------------------------------------- | --------------------- |
| Cold startup         | < 100ms | ~500ms   | `time bun src/main.tsx --version`            |
| Routing latency      | < 1ms   | < 1ms ✅ | `performance.now()` around `route()`         |
| System prompt tokens | < 600   | ~8,000   | `estimateTokens(buildSystemPrompt())`        |
| Binary size          | < 5MB   | ~50MB    | `ls -la quantum` after `bun build --compile` |
| Runtime deps         | < 15    | 74       | `jq '.dependencies                           | length' package.json` |
| Memory (idle)        | < 50MB  | ~150MB   | `process.memoryUsage().heapUsed`             |

### Quality Metrics

| Metric                     | Target               | How to Measure                      |
| -------------------------- | -------------------- | ----------------------------------- |
| Intent accuracy            | > 90%                | Test suite with labeled prompts     |
| Mode selection correctness | > 85%                | Manual audit of 100 diverse prompts |
| Token efficiency           | < 5% system overhead | `systemTokens / totalTokens * 100`  |
| Tool call precision        | > 80%                | Ratio of useful tool calls vs total |

---

## Architecture Notes

### Design Principles

1. **Router is a policy engine** — layered decisions, not a switchboard
2. **Routing is deterministic** — regex + keyword scoring, no LLM needed for routing
3. **Mode switches are cheap** — preserve only `keyFiles`, `findings`, `taskDescription`, `decisions`
4. **Memory is intentional** — loaded by relevance threshold, not blindly
5. **Rust is optional** — graceful degradation to TypeScript fallbacks
6. **Prompts are budget-conscious** — every token in the system prompt must earn its place

### Key Files Reference

| File                      | Purpose                                              | Lines |
| ------------------------- | ---------------------------------------------------- | ----: |
| `Router.ts`               | Central dispatch — `route()` orchestrates all layers |   362 |
| `TaskAnalyzer.ts`         | Intent classification + complexity scoring           |   644 |
| `ModeManager.ts`          | State machine + prompt mods + context preservation   |   722 |
| `ModelSelector.ts`        | Tier selection + escalation + cost awareness         |   470 |
| `ToolPolicyManager.ts`    | Per-intent tool policies + activation levels         |   505 |
| `ContextStrategy.ts`      | Budget allocation + compression + prioritization     |   522 |
| `MemoryStrategy.ts`       | Relevance scoring + intent-aware memory loading      |   486 |
| `TokenBudgetTracker.ts`   | Usage tracking + enforcement + auto-trimming         |   496 |
| `ParallelToolExecutor.ts` | Dependency graph + safety-based parallelism          |   581 |
| `rust/src/router.rs`      | Hot-path prompt analysis (Rust + RegexSet)           |   342 |
| `src/rust/bindings.ts`    | NAPI-RS bindings with JS fallbacks                   |   376 |

---

## Recent Changes

### 2026-04-11

- Full codebase audit completed — all TODO items verified against actual source files
- Created 4 research documents: architecture, router deep-dive, skills/optimization, roadmap
- Added Phase 7 (Quantum Code Port) with detailed implementation tasks
- Added Router ↔ QueryEngine integration blocker (Phase 5.1)
- Updated metrics table with current vs target values
- Added key files reference table

### 2024-04-06

- Fixed typo in TaskAnalyzer.ts: `SIMPLICITY_INDICITY` → `SIMPLICITY_INDICATORS`
- Updated TODO to reflect completed Phase 1 and Phase 2 implementation
- All core router components implemented
