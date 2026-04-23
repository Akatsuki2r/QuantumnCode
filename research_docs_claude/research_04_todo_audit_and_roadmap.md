# Research 04 — TODO Audit, Porting Roadmap & Implementation Blueprint

> Complete audit of the QUANTUM_TODO.md, mapping every item to its implementation status in the actual codebase, with a revised porting roadmap for Quantum Code.

---

## 1. QUANTUM_TODO.md — Full Audit

### Phase 1: Core Router Foundation — ✅ COMPLETE

| Item | Status | Actual Implementation |
|------|:------:|----------------------|
| Router module structure | ✅ | `src/router/` — 9 files |
| TaskAnalyzer interfaces | ✅ | `src/router/types.rs` — `RoutingDecision`, `RouterConfig` |
| Routing decision types | ✅ | `src/router/types.rs` — 10+ fields |
| Router config schema | ✅ | `src/router/types.rs` — `RouterConfig` with defaults |
| Intent classifier | ✅ | `src/router/analyzer.rs` — regex-based, 12+ intents |
| Intent categories | ✅ | `src/router/types.rs` — 5 categories |
| Signal extraction | ✅ | `src/router/analyzer.rs` — complexity + tool patterns |
| Complexity estimation | ✅ | `src/router/analyzer.rs` — weighted scoring, 5 levels |
| Mode state machine | ✅ | `src/prompts/modes.rs` — Plan/Build/Chat modes |
| Mode-specific prompts | ✅ | `src/prompts/modes.rs` — mode-specific prompts |
| Mode switching logic | ✅ | `src/router/mode.rs` — mode transitions |
| Mode persistence | ⚠️ PARTIAL | Session persists in memory, not disk yet |

### Phase 2: Model & Tool Selection — ✅ COMPLETE

| Item | Status | Actual Implementation |
|------|:------:|----------------------|
| Model tier interfaces | ✅ | `src/router/model.rs` — tier selection |
| Complexity → tier mapping | ✅ | `src/router/model.rs` — tier selection |
| Escalation/de-escalation | ✅ | `src/router/model.rs` — `upgradeTier()`, `downgradeTier()` |
| Cost-aware selection | ✅ | `src/router/model.rs` — cost estimation |
| Tool necessity estimation | ✅ | `src/router/tools.rs` — pattern-based |
| Tool filtering by task | ✅ | `src/router/tools.rs` — per-mode policies |
| Batch tool planning | ⚠️ PARTIAL | Sequential execution only, no batching |
| Tool sparseness optimization | ⚠️ PARTIAL | Policy defined, limited activation levels |
| Context relevance scoring | ✅ | `src/rag/` — keyword matching retriever |
| Context compression | ✅ | `src/rag/compact_prompts.rs` — compression |
| Working vs persistent memory | ⚠️ PARTIAL | Session memory exists, not project memory |
| Context budget management | ✅ | `src/router/context.rs` — budget tiers |
| Router test suite | ✅ | `src/router/tests.rs` — 100+ tests |

### Phase 3: Execution Optimization — ⚠️ PARTIAL

| Item | Status | Actual Implementation |
|------|:------:|----------------------|
| Parallel tool execution | ⚠️ PARTIAL | Sequential only, no parallel batching |
| Predictive tool pre-loading | ❌ | Not implemented |
| Response caching | ❌ | Not implemented |
| Aggressive context trimming | ⚠️ PARTIAL | Prompt compaction exists, not auto-triggered |
| Summary injection points | ⚠️ PARTIAL | Memory system exists, not wired to prompt assembly |
| Minimal prompt assembly | ✅ | Mode-specific prompts, compact formatting |
| Token budget tracking | ✅ | `src/router/context.rs` — full implementation |
| Relevance-based memory loading | ⚠️ PARTIAL | RAG exists but not context-aware |
| Memory write-back policies | ❌ | Memory history kept in session only |
| Session vs project memory | ❌ | Types defined, separation not enforced |
| Memory expiration | ❌ | Not implemented |

### Phase 4: Rust Integration — ✅ COMPLETE

| Item | Status | Actual Implementation |
|------|:------:|----------------------|
| Cargo workspace | ✅ | Single `Cargo.toml`, all in `src/` |
| Rust source files | ✅ | `src/router/`, `src/providers/`, `src/tools/` |
| File indexing (glob) | ✅ | `src/tools/glob.rs` — WalkDir-based |
| Grep (content search) | ✅ | `src/tools/grep.rs` — regex matching |
| Router hot paths | ✅ | `src/router/analyzer.rs` — regex-based analysis |
| Token estimation | ✅ | `src/router/context.rs` — character-based estimate |
| Rust test suite | ✅ | Multiple `#[cfg(test)]` modules, 150+ tests |
| Performance regression tests | ❌ | Not implemented |

### Phase 5: Mode Implementation — ❌ NOT STARTED

All mode implementation items (Plan, Build, Chat execution paths) are **defined but not wired** to the main query engine. `ModeManager` exists but `QueryEngine.ts` doesn't consume it.

| Item | Status | Notes |
|------|:------:|-------|
| Plan-only execution path | ❌ | Mode definitions exist but not integrated |
| Structured plan output | ❌ | Prompt mods defined but not enforced |
| Plan → build transition | ❌ | State machine ready, no trigger logic |
| Build progress checkpoints | ❌ | Not implemented |
| Incremental execution | ❌ | Not implemented |
| Chat minimal tools | ❌ | Policy defined, not enforced in query loop |
| Chat → plan escalation | ❌ | Not implemented |

### Phase 6: Quality & Testing — ❌ MOSTLY NOT STARTED

| Item | Status | Notes |
|------|:------:|-------|
| Router decision tests | ✅ | `src/router/__tests__/` exists |
| Mode transition tests | ❌ | Not found |
| Tool policy tests | ❌ | Not found |
| Context strategy tests | ❌ | Not found |
| E2E routing tests | ❌ | Not found |
| Architecture docs | ❌ | This research addresses it |
| Performance tuning guide | ❌ | This research addresses it |

---

## 2. Integration Gap: Router ↔ QueryEngine

The **biggest gap** in the current codebase is that the router exists as a standalone module but is **not connected to the QueryEngine** (the 46K-line core that actually calls the LLM).

### What's Missing

```
Current flow:
  User Input → QueryEngine.ts → LLM API → Tool Loop → Output
                    ↑
                    ╳  Router is NOT consulted
                    
Required flow:
  User Input → Router.route() → RoutingDecision
                                       │
                                       ▼
                               QueryEngine.ts
                                   │ Uses: mode, model tier, tool policy,
                                   │       context strategy, memory policy
                                   ▼
                               LLM API → Tool Loop → Output
```

### Integration Points Needed

1. **Pre-query routing**: Before calling the LLM, `router.route(prompt, cwd, tools)` must be called
2. **System prompt assembly**: Mode-specific prompts from `ModeManager` must be injected
3. **Tool filtering**: `ToolPolicyManager.isToolAllowed()` must gate which tools are sent to the LLM
4. **Context budgeting**: `ContextStrategy.allocateBudget()` must limit conversation history
5. **Token tracking**: `TokenBudgetTracker.updateUsage()` must be called after each turn
6. **Mode transitions**: Router must be able to transition modes mid-conversation

---

## 3. Quantum Code Porting Roadmap

### Phase A: Foundation (Week 1-2)

#### A1: Core Query Pipeline
- [ ] Create minimal `query.ts` (~2K lines max)
- [ ] Support streaming responses from Anthropic API (or local model)
- [ ] Implement tool-call loop (execute → feed result → repeat)
- [ ] Support both API and local inference backends

#### A2: Router Integration
- [ ] Port `src/router/` with simplifications (see Research 02)
- [ ] Wire `route()` into query pipeline before every LLM call
- [ ] Use routing decision to select: model, tools, context budget
- [ ] Implement mode-aware system prompt assembly

#### A3: Core Tools (8 tools)
- [ ] `read` — FileRead (read file, return content)
- [ ] `edit` — FileEdit (find-and-replace in file)
- [ ] `write` — FileWrite (create/overwrite file)
- [ ] `bash` — Run shell commands
- [ ] `glob` — Find files by pattern
- [ ] `grep` — Search file contents
- [ ] `search` — Web search
- [ ] `ask` — Ask user for clarification

#### A4: Configuration
- [ ] Simple JSON config at `~/.quantum/config.json`
- [ ] Project-level config at `.quantum/config.json`
- [ ] Environment variables for API keys
- [ ] No Zod schemas initially — use TypeScript types + runtime checks

### Phase B: Intelligence Layer (Week 3-4)

#### B1: Skill System
- [ ] Minimal skill loader (`skills.ts` < 100 lines)
- [ ] Load from `.quantum/skills/*/SKILL.md`
- [ ] Lazy content loading (frontmatter parsed on startup, body on invoke)
- [ ] 5 bundled skills: `commit`, `review`, `debug`, `test`, `plan`

#### B2: Context & Memory
- [ ] Git context injection (branch, status, recent commits)
- [ ] QUANTUM.md memory file (project-level conventions)
- [ ] Session memory (key decisions carried between turns)
- [ ] Relevance-based memory trimming per router decision

#### B3: Mode System
- [ ] Implement chat → plan → build transitions in query loop
- [ ] Mode-specific output formatting
- [ ] Automatic mode suggestion based on router analysis
- [ ] Mode indicator in prompt display

### Phase C: Performance (Week 5-6)

#### C1: Rust Native Module
- [ ] Port existing `rust/` workspace
- [ ] Build for target platforms (Linux x86_64, ARM64, macOS)
- [ ] Wire into TypeScript via NAPI-RS bindings
- [ ] Benchmark: glob, grep, token estimation, prompt analysis

#### C2: Local Model Support
- [ ] `llama.cpp` server integration (HTTP API)
- [ ] Ollama integration (HTTP API)
- [ ] GGUF model management (download, list, select)
- [ ] KV cache persistence for multi-turn conversations
- [ ] Quantization-aware prompt formatting

#### C3: Optimization
- [ ] Parallel tool execution (from router's `ParallelToolExecutor` logic)
- [ ] Aggressive context trimming (80% threshold → compress to 60%)
- [ ] Response caching for repeated queries (LRU cache)
- [ ] Startup profiling (target < 100ms cold start)

### Phase D: Polish (Week 7-8)

#### D1: CLI Interface
- [ ] Minimal REPL (no React/Ink — use `readline` or `rustyline` via FFI)
- [ ] Syntax-highlighted output (using `picocolors` or ANSI directly)
- [ ] Slash commands: `/mode`, `/cost`, `/skills`, `/config`, `/compact`
- [ ] Streaming output with live typing effect

#### D2: Testing
- [ ] Router unit tests (intent classification, complexity estimation)
- [ ] Tool integration tests (each tool with mock filesystem)
- [ ] E2E test: prompt → route → query → response
- [ ] Performance benchmarks (startup time, routing latency, token efficiency)

---

## 4. Key Implementation Decisions

### Decision 1: Single Query Function vs QueryEngine Class

**Recommendation**: Single async generator function

```typescript
// Instead of a 46K-line class:
export async function* query(
  prompt: string,
  config: QueryConfig,
  tools: Tool[],
): AsyncGenerator<StreamChunk> {
  const decision = route(prompt, config.cwd)
  const systemPrompt = buildSystemPrompt(decision, config)
  const filteredTools = filterTools(tools, decision.tools)
  
  for await (const chunk of callLLM(systemPrompt, prompt, filteredTools, config)) {
    if (chunk.type === 'tool_call') {
      const result = await executeTool(chunk.tool, chunk.params)
      yield { type: 'tool_result', result }
      // Feed result back and continue
    } else {
      yield chunk  // Text streaming
    }
  }
}
```

### Decision 2: API Provider Abstraction

Support multiple backends behind a single interface:

```typescript
interface LLMProvider {
  // Streaming chat completion
  chat(messages: Message[], tools: ToolDef[]): AsyncGenerator<Chunk>
  // Check if model supports feature
  supports(feature: 'tools' | 'thinking' | 'streaming'): boolean
  // Estimate cost
  estimateCost(inputTokens: number, outputTokens: number): number
}

// Implementations:
class AnthropicProvider implements LLMProvider { /* ... */ }
class OllamaProvider implements LLMProvider { /* ... */ }
class LlamaCppProvider implements LLMProvider { /* ... */ }
class OpenAIProvider implements LLMProvider { /* ... */ }
```

### Decision 3: Tool Schema Format

Use a minimal tool schema that works with both Anthropic API and local models:

```typescript
interface ToolDef {
  name: string
  description: string  // Max 100 chars
  parameters: Record<string, {
    type: 'string' | 'number' | 'boolean'
    description: string  // Max 50 chars
    required?: boolean
  }>
}
```

### Decision 4: State Management

**No React, no stores, no singletons.**

```typescript
// Session state is a plain object, threaded through function calls
interface Session {
  id: string
  mode: ExecutionMode
  messages: Message[]
  tokenUsage: { input: number; output: number }
  activeFiles: string[]
  findings: string[]
  config: ResolvedConfig
}

// Create fresh session
function createSession(config: ResolvedConfig): Session { /* ... */ }

// Update session (immutable-style)
function addMessage(session: Session, msg: Message): Session {
  return { ...session, messages: [...session.messages, msg] }
}
```

---

## 5. Dependency Manifest for Quantum Code

### Runtime Dependencies (Target: 12)

| Package | Size | Purpose |
|---------|-----:|---------|
| `@anthropic-ai/sdk` | 200KB | Anthropic API client |
| `zod` | 50KB | Schema validation (tool params) |
| `picocolors` | 3KB | Terminal colors (replaces chalk) |
| `better-sqlite3` | 500KB | Local state/cache (native) |
| `ignore` | 20KB | .gitignore parsing |
| `yaml` | 80KB | SKILL.md frontmatter parsing |
| `diff` | 50KB | File diff generation |
| `semver` | 15KB | Version comparison |
| `undici` | 300KB | HTTP client (if not using native fetch) |
| `ws` | 50KB | WebSocket (for local model server) |
| Native: Rust NAPI | — | Glob, grep, token estimation |
| Native: Bun runtime | — | JSX, bundling, test runner |

**Total: ~1.3MB** vs original ~8MB+ for 74 dependencies.

### Dev Dependencies (Target: 5)

| Package | Purpose |
|---------|---------|
| `typescript` | Type checking |
| `vitest` | Test runner |
| `esbuild` | Bundler |
| `@types/node` | Node.js types |
| `@types/better-sqlite3` | SQLite types |

---

## 6. Verification Plan

### Automated Tests

```bash
# Unit tests
bun test src/router/    # Router classification, complexity, mode transitions
bun test src/tools/     # Each tool in isolation
bun test src/query.ts   # Query pipeline with mocked LLM

# Integration tests
bun test tests/e2e/     # Full prompt → response cycles

# Performance
bun test tests/perf/    # Startup < 100ms, routing < 1ms, tool schemas < 200 tokens
```

### Benchmarks

| Metric | Target | How to Measure |
|--------|--------|---------------|
| Cold start | < 100ms | `time bun src/main.ts --version` |
| Routing latency | < 1ms | `performance.now()` around `route()` |
| System prompt tokens | < 600 | `estimateTokens(buildSystemPrompt())` |
| Binary size | < 5MB | `ls -la quantum` after `bun build --compile` |
| Memory (idle) | < 50MB | `process.memoryUsage().heapUsed` |
| Memory (active) | < 200MB | During tool execution |

---

## 7. Summary: What Changes vs What Stays

### KEEP (Core Value)

| Component | Reason |
|-----------|--------|
| Router pipeline (7 layers) | Sound architecture, well-typed |
| Regex-based intent classification | Fast, no LLM needed, < 1ms |
| Mode state machine | Clean transitions, preserved context |
| Rust acceleration module | 10-50x speedup on hot paths |
| Tool safety classifications | Essential for parallel execution |
| Token budget tracking | Critical for small-model contexts |

### REDESIGN (Reduce Bloat)

| Component | Original | Quantum Code |
|-----------|----------|-------------|
| 7 manager classes + singletons | OOP + singletons | Pure functions |
| 40 tools | Full catalog | 8 core tools |
| 16 bundled skills | Feature-rich | 5 essential skills |
| Coordinator prompt (4K tokens) | Verbose examples | 500-token compressed |
| System prompt (8K tokens) | Prose-heavy | 600-token structured |
| Skill loader (1,088 lines) | 5 source paths, dedup | Single path, 100 lines |

### DROP (Unnecessary for v1)

| Component | Why |
|-----------|-----|
| React/Ink UI (140 components) | Plain terminal output |
| Bridge (IDE integration) | CLI-first |
| 74 npm dependencies | Replace with native/minimal |
| GrowthBook analytics | Not needed |
| Voice system | Niche feature |
| Plugin marketplace | Over-engineered |
| Remote sessions | Enterprise |
| Server mode (Drizzle/Postgres) | Local-first |
