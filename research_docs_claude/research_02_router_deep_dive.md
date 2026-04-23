# Research 02 — Router Deep Dive: Architecture, Implementation & Optimization

> Complete technical breakdown of the Quantum Code router — how it classifies tasks, selects models, manages modes, policies, and context — with optimization strategies for smaller models and consumer hardware.

---

## 1. Router Philosophy

The router is a **policy engine**, not a simple switchboard. It makes layered decisions that cascade through seven stages:

```
User Prompt
    │
    ▼
┌─────────────────┐
│ 1. Intent        │  What kind of task?  (chat, fix, implement, review…)
│    Classification│
└────────┬────────┘
         ▼
┌─────────────────┐
│ 2. Complexity    │  How hard? (trivial → simple → moderate → complex → heavy)
│    Estimation    │
└────────┬────────┘
         ▼
┌─────────────────┐
│ 3. Mode          │  What execution mode? (chat, plan, build, review, debug)
│    Selection     │
└────────┬────────┘
         ▼
┌─────────────────┐
│ 4. Model Tier    │  What capability level? (local, fast, standard, capable)
│    Selection     │
└────────┬────────┘
         ▼
┌─────────────────┐
│ 5. Tool Policy   │  Which tools? (allowed, disallowed, batch settings)
│    Determination │
└────────┬────────┘
         ▼
┌─────────────────┐
│ 6. Context       │  How much context to load? (minimal → comprehensive)
│    Strategy      │
└────────┬────────┘
         ▼
┌─────────────────┐
│ 7. Memory Policy │  What memory to load? (project, user, session)
└────────┬────────┘
         ▼
    RoutingDecision
```

---

## 2. File-by-File Breakdown

### `src/router/` — 12 files, ~148KB total

| File | Lines | Purpose |
|------|------:|---------|
| `types.ts` | 404 | All type definitions, constants, default config |
| `Router.ts` | 362 | Central router class — `route()` method drives the pipeline |
| `TaskAnalyzer.ts` | 644 | Intent classification via regex patterns + complexity scoring |
| `ModeManager.ts` | 722 | Mode state machine with persistence, prompt mods per mode |
| `ModelSelector.ts` | 470 | Model tier selection with escalation/de-escalation |
| `ToolPolicyManager.ts` | 505 | Per-intent tool policies, safety classifications |
| `ContextStrategy.ts` | 522 | Context budget allocation and prioritization |
| `MemoryStrategy.ts` | 486 | Memory loading with relevance scoring per intent |
| `TokenBudgetTracker.ts` | 496 | Token usage tracking, budget enforcement, auto-trimming |
| `ParallelToolExecutor.ts` | 581 | Dependency graph-based parallel tool execution |
| `ModePersistence.ts` | ~200 | Session-level mode state serialization |
| `index.ts` | 246 | Public API exports + convenience functions |

---

## 3. Intent Classification — How It Works

**Location**: `src/router/TaskAnalyzer.ts`

The `TaskAnalyzer` uses **regex-based pattern matching** against the raw user prompt. No LLM call is needed — this runs in **< 1ms** on CPU.

### Intent Categories (16 intents, 5 categories)

| Category | Intents |
|----------|---------|
| **Conversational** | `chat`, `explain`, `summarize` |
| **Analytical** | `plan`, `analyze`, `review` |
| **Implementation** | `implement`, `refactor`, `fix`, `test` |
| **Operational** | `search`, `inspect`, `execute`, `debug` |
| **Configuration** | `configure` |

### Pattern Matching (Actual Code)

Each intent has an array of `RegExp` patterns with weights. The analyzer scores all 16 intents against the prompt and picks the highest:

```typescript
// Simplified from TaskAnalyzer.ts
const INTENT_PATTERNS = {
  fix: {
    patterns: [
      /^(fix|bug|error|issue|problem|broken)/i,    // Starts with fix-related word
      /\b(not working|doesn't work|fails)\b/i,     // Contains failure language
      /\b(crash|throw|undefined|null pointer)\b/i,  // Contains error types
    ],
    weight: 0.95,  // High confidence when matched
  },
  chat: {
    patterns: [
      /^(what is|how do|why does|can you explain)/i,
      /^(hi|hello|hey)/i,
    ],
    weight: 1.0,
  },
  // ... 14 more intents
}
```

### Scoring Algorithm

```
for each intent:
    score = 0
    for each pattern in intent.patterns:
        if pattern.test(prompt):
            score += intent.weight
    scores[intent] = score

bestIntent = max(scores)
confidence = min(maxScore / 2, 1.0)
```

### Critical Optimization Point

The regex patterns are compiled at module load time. In Rust (`rust/src/router.rs`), they use `lazy_static!` + `regex::RegexSet` for **compiled, zero-allocation matching**:

```rust
lazy_static! {
    static ref INTENT_PATTERNS: IntentPatterns = IntentPatterns::new();
}
// RegexSet matches ALL patterns in a single pass over the input
```

---

## 4. Complexity Estimation

**Location**: `TaskAnalyzer.ts:335-491`

Complexity is scored on a weighted point system:

| Signal | Points |
|--------|--------|
| `fileScope > 10` | +30 |
| `fileScope > 5` | +15 |
| `editScope > 100` | +25 |
| `editScope > 50` | +15 |
| `operationCount > 5` | +20 |
| `riskLevel === 'high'` | +20 |
| `requiresCoordination` | +10 |
| `hasDependencies` | +10 |
| `reasoningDepth === 'deep'` | +15 |

### Score → Level Mapping

| Score | Level |
|-------|-------|
| < 15 | `trivial` |
| < 35 | `simple` |
| < 60 | `moderate` |
| < 90 | `complex` |
| ≥ 90 | `heavy` |

### Complexity Indicator Keywords (Regex)

```typescript
const COMPLEXITY_INDICATORS = {
  multiFile:      /\b(all files|multiple files|across|throughout|every)\b/i,
  entireCodebase: /\b(entire|whole|all|complete|full)\b.*\b(codebase|project|repo)\b/i,
  coordination:   /\b(then|after|before|while|simultaneously|parallel)\b/i,
  dependencies:   /\b(depend(s|ent|encies)?|require(s)?|must|should|before)\b/i,
  largeScope:     /\b(refactor|rewrite|restructure|migrate|port)\b/i,
  deepReasoning:  /\b(design|architect|strategy|approach|consider|trade.?off)\b/i,
  highRisk:       /\b(careful|caution|risk|breaking|critical|production)\b/i,
  destructive:    /\b(delete|remove|drop|destructive|irreversible)\b/i,
}

const SIMPLICITY_INDICATORS = {
  singleFile: /\b(this file|this function|single|one)\b/i,
  quick:      /\b(quick|simple|small|minor|just)\b/i,
  read:       /\b(read|show|display|print|cat|view)\b/i,
}
```

---

## 5. Mode Management (State Machine)

**Location**: `src/router/ModeManager.ts`

### Five Execution Modes

| Mode | Writes? | Tool Access | Prompt Shape |
|------|---------|-------------|--------------|
| `chat` | ❌ | Minimal | Concise, suggest rather than execute |
| `plan` | ❌ | Read-only + Tasks | Structured plan output (Analysis → Approach → Steps → Risks) |
| `build` | ✅ | Full | Implementation-focused, atomic changes |
| `review` | ❌ | Read-only | Structured review (Summary → Issues → Suggestions) |
| `debug` | ❌ | Read + Bash | Systematic investigation (Error → Root Cause → Fix Suggestions) |

### Valid Transitions

```
chat ──→ plan, build, debug
plan ──→ build, review, chat
build ──→ review, debug, chat
review ──→ build, plan, chat
debug ──→ build, plan, chat
```

### Mode State Tracking

```typescript
interface ModeState {
  currentMode: ExecutionMode
  previousMode: ExecutionMode | null
  turnCount: number
  enteredAt: number
  enterReason: string
  preservedContext: {
    keyFiles: string[]       // Max 20 files
    findings: string[]       // Max 10 findings
    taskDescription: string
    decisions: string[]      // Max 10 decisions
  }
}
```

### Context Preservation on Transition

When switching modes, the `preserveContextForTransition()` method trims carried-over state based on the **outgoing** mode's preservation settings:

| Mode | Keep Tool Results | Keep File Cache | Keep Memory | Max Preserved Turns |
|------|:-:|:-:|:-:|----:|
| chat | ❌ | ✅ | ❌ | 2 |
| plan | ✅ | ✅ | ✅ | 10 |
| build | ✅ | ✅ | ✅ | 50 |
| review | ✅ | ✅ | ✅ | 20 |
| debug | ✅ | ✅ | ✅ | 30 |

### Persistence

`ModeManagerWithPersistence` extends `ModeManager` with debounced disk writes (500ms) to the session directory. State survives process restarts.

---

## 6. Model Tier Selection

**Location**: `src/router/ModelSelector.ts`

### Four Model Tiers

| Tier | Class | Context | Cost | Latency | Reasoning |
|------|-------|--------:|-----:|---------|-----------|
| `local` | Local LLM | 32K | $0 | Fast | Shallow |
| `fast` | Haiku | 200K | 0.25x | Fast | Shallow |
| `standard` | Sonnet | 200K | 1.0x | Moderate | Moderate |
| `capable` | Opus | 200K+ | 3.0x | Slow | Deep |

### Selection Algorithm

```
1. Base tier = complexityTierMap[complexity]
   trivial/simple → fast
   moderate/complex → standard
   heavy → capable

2. Adjust for reasoning depth:
   deep → upgrade one tier

3. Adjust for precision:
   precision_required && fast → standard

4. Adjust for risk:
   high → capable
   medium && fast → standard

5. Adjust for security:
   security_sensitive → capable

6. Adjust for speed:
   speed_priority → downgrade one tier

7. Apply user preference override

8. Check cost budget (downgrade if over)
```

### Escalation Detection

Each tier defines `escalationTriggers` — string patterns that indicate the task outgrows the current tier:

```typescript
fast.escalationTriggers = [
  'complex code changes',
  'architecture decisions',
  'multi-file coordination',
]
```

---

## 7. Tool Policy System

**Location**: `src/router/ToolPolicyManager.ts`

### Per-Intent Tool Policies

Each of the 16 intents has explicit `allowed`, `disallowed`, and `requiresApproval` tool lists:

```typescript
// Example: 'fix' intent
fix: {
  allowed: ['FileRead', 'Grep', 'FileEdit', 'Bash'],
  disallowed: [],
  requiresApproval: ['FileWrite', 'Agent'],
}

// Example: 'chat' intent
chat: {
  allowed: ['AskUserQuestion'],
  disallowed: ['FileEdit', 'FileWrite', 'Bash', 'Agent'],
  requiresApproval: ['FileRead', 'Glob', 'Grep'],
}
```

### Tool Safety Levels

| Level | Tools |
|-------|-------|
| `safe` (read-only) | FileRead, Glob, Grep, WebFetch, WebSearch, TaskList, TaskGet |
| `moderate` (write) | FileEdit, FileWrite, NotebookEdit, Agent |
| `risky` (execute) | Bash, TaskCreate, TaskUpdate, AskUserQuestion |

### Tool Activation by Mode

| Mode | Activation Level |
|------|------------------|
| `chat` | `minimal` — only essential tools |
| `plan` | `standard` — common tools |
| `build` | `full` — all tools |
| `review` | `standard` |
| `debug` | `standard` |

---

## 8. Context Budget System

**Location**: `src/router/ContextStrategy.ts`

### Budget Allocations by Strategy

| Strategy | Max Tokens | System | History | Memory | Tool Results |
|----------|----------:|-------:|--------:|-------:|-------------:|
| `minimal` | 4,000 | 1,000 | 1,000 | 500 | 1,500 |
| `relevant` | 16,000 | 2,000 | 6,000 | 2,000 | 6,000 |
| `standard` | 50,000 | 4,000 | 20,000 | 5,000 | 21,000 |
| `comprehensive` | 100,000 | 6,000 | 40,000 | 10,000 | 44,000 |

### Strategy Selection Logic

```
trivial          → minimal
simple + readOnly → relevant
simple + !readOnly → minimal
moderate         → standard
complex/heavy    → standard (or comprehensive if fileScope > 10)
```

### Compression Triggers

- **85% capacity**: Suggest trimming, target 60%
- **100% capacity**: Force trim, target 50%
- Low-priority, low-relevance items are trimmed first

---

## 9. Parallel Tool Execution

**Location**: `src/router/ParallelToolExecutor.ts`

### Safety Classifications

| Classification | Can Parallelize? | Tools |
|----------------|:---:|-------|
| `safe` | ✅ Always | FileRead, Glob, Grep, WebFetch, WebSearch |
| `isolated` | ⚠️ If no file overlap | FileEdit, FileWrite, NotebookEdit |
| `sequential` | ❌ Never | Bash, Agent, TaskCreate, AskUserQuestion |

### Execution Algorithm

Uses an `ExecutionGraph` (DAG of tool requests with dependencies):

```
1. Build dependency graph from requests
2. Find all "ready" nodes (no unresolved dependencies)
3. Execute ready nodes in parallel (up to maxConcurrent)
4. On completion, resolve dependents
5. Repeat until graph is complete
```

### File Overlap Detection

Two `isolated`-class tools can run in parallel IF their file paths don't overlap:

```typescript
function hasFileOverlap(params1, params2): boolean {
  const path1 = getPathFromParams(params1)  // checks .path, .file_path, .filePath, .directory
  const path2 = getPathFromParams(params2)
  return path1 === path2 ||
         path1.startsWith(path2 + '/') ||
         path2.startsWith(path1 + '/')
}
```

---

## 10. Rust Acceleration Layer

### Architecture

```
TypeScript (src/router/)
    │ High-level routing decisions
    │
    ▼
src/rust/bindings.ts
    │ Lazy-loaded NAPI-RS bindings (optional fallback to JS)
    │
    ▼
rust/src/ (Cargo workspace)
    ├── lib.rs            # NAPI exports
    ├── router.rs         # Hot-path prompt analysis (lazy_static RegexSet)
    ├── token_estimate.rs # Fast token counting
    └── file_ops.rs       # Glob + grep via walkdir + regex
```

### Performance: Rust vs TypeScript

| Operation | TypeScript | Rust | Speedup |
|-----------|-----------|------|---------|
| Prompt analysis (regex) | ~2ms | ~0.05ms | 40x |
| Token estimation (100K chars) | ~5ms | ~0.1ms | 50x |
| Glob search (10K files) | ~200ms | ~15ms | 13x |
| Grep search (1K files) | ~500ms | ~30ms | 17x |

### Graceful Degradation

The Rust module is **optional**. If the `.node` binary isn't built, all operations fall back to pure TypeScript:

```typescript
async function initRustModule(): Promise<void> {
  try {
    rustModule = await import('../../rust/index.node')
  } catch {
    rustModule = null  // Falls back to JS implementations
  }
}
```

---

## 11. Implementing the Router in Quantum Code

### Minimal Router (8 files, ~3K lines target)

```
quantum-code/src/router/
├── types.ts         # Lean types (< 200 lines)
├── router.ts        # Core router (< 300 lines)
├── analyzer.ts      # Intent + complexity (< 400 lines)
├── mode.ts          # Mode state machine (< 300 lines)
├── model.ts         # Model tier selection (< 200 lines)
├── tools.ts         # Tool policy (< 300 lines)
├── context.ts       # Context budget (< 200 lines)
├── index.ts         # Exports
```

### Key Simplifications

1. **Merge MemoryStrategy into ContextStrategy** — same budget mechanism, less indirection
2. **Remove ModePersistence** — store mode in a single JSON key in the session
3. **Inline ParallelToolExecutor** — move to the query engine; the router shouldn't own execution
4. **Flatten types** — combine `ToolPolicy` fields into the `RoutingDecision` directly
5. **Remove TokenBudgetManager** (singleton tracker per session) — use a plain counter on the session object

### Revised Router Flow

```typescript
// Quantum Code: simplified routing in < 100 lines
export function route(prompt: string, cwd: string): RoutingDecision {
  const intent = classifyIntent(prompt)    // 1 function, regex-based
  const complexity = estimateComplexity(prompt) // 1 function, keyword scoring
  const mode = selectMode(intent, complexity)
  const modelTier = selectModel(complexity, intent)
  const tools = selectTools(intent, mode)
  const contextBudget = selectContext(complexity)

  return { intent, complexity, mode, modelTier, tools, contextBudget }
}
```

---

## 12. Improved Router Design for Quantum Code

### Problem with Current Design
The current router has **too many classes with singletons exported**. Each of `Router`, `ModeManager`, `ModelSelector`, `ToolPolicyManager`, `ContextStrategyManager`, `MemoryStrategyManager`, `TokenBudgetTracker`, `TokenBudgetManager`, `ParallelToolExecutor` exports both a class AND a singleton AND a factory. This triples the API surface.

### Proposed: Functional Router

```typescript
// router.ts — single file, no classes, no singletons

import type { RoutingDecision, RouterConfig } from './types'

const DEFAULT_CONFIG: RouterConfig = { /* ... */ }

export function route(
  prompt: string,
  config: RouterConfig = DEFAULT_CONFIG
): RoutingDecision {
  const intent = classifyIntent(prompt)
  const complexity = scoreComplexity(prompt)
  const mode = pickMode(intent, complexity)
  const model = pickModel(complexity, intent, config)
  const tools = pickTools(intent, mode)
  const budget = pickBudget(complexity)

  return {
    intent, complexity, mode, model, tools, budget,
    readOnly: READ_ONLY_INTENTS.has(intent),
    confidence: intent.confidence,
  }
}

// Pure functions — no state, no classes
function classifyIntent(prompt: string): Intent { /* regex scoring */ }
function scoreComplexity(prompt: string): Complexity { /* keyword scoring */ }
function pickMode(intent: Intent, complexity: Complexity): Mode { /* lookup */ }
function pickModel(c: Complexity, i: Intent, cfg: RouterConfig): ModelTier { /* rules */ }
function pickTools(intent: Intent, mode: Mode): ToolPolicy { /* lookup */ }
function pickBudget(complexity: Complexity): ContextBudget { /* lookup */ }
```

### Benefits
- **Zero allocations** per routing call (no `new` constructors)
- **Tree-shakeable** (unused functions are eliminated)
- **Testable** (pure functions, no mocked singletons)
- **< 500 lines** total
