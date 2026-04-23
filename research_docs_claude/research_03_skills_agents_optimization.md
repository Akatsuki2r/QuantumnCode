# Research 03 — Skills, Agents, Coordinator & Optimized Alternatives

> How the original skill system, agent/coordinator orchestration, and tool system work — with redesigned, token-efficient alternatives for Quantum Code targeting smaller models on consumer hardware.

---

## 1. Original Skill System

### Architecture

Skills are **reusable named workflows** that bundle a prompt (markdown content) with tool permissions, model overrides, and hook configurations.

```
User types: /commit
    │
    ▼
SkillTool OR slash command lookup
    │
    ▼
loadSkillsDir.ts → loads SKILL.md from:
  1. ~/.claude/skills/          (user)
  2. .claude/skills/            (project)
  3. managed policy path        (enterprise)
  4. bundled in binary          (built-in)
    │
    ▼
parseFrontmatter() → extracts YAML header:
  - name, description, when_to_use
  - allowed-tools, model, effort
  - hooks, paths, argument-hint
  - user-invocable, disable-model-invocation
    │
    ▼
createSkillCommand() → wraps as a PromptCommand
    │
    ▼
getPromptForCommand(args, context) → returns ContentBlockParam[]
  - Substitutes ${1}, ${2} arguments
  - Replaces ${CLAUDE_SKILL_DIR}, ${CLAUDE_SESSION_ID}
  - Executes inline shell commands (!`...`) for non-MCP skills
  - Prepends "Base directory for this skill: <dir>"
```

### Skill File Format (SKILL.md)

```markdown
---
name: commit
description: Create a git commit with a generated message
allowed-tools: ["Bash(git *)"]
when_to_use: "When the user wants to commit changes"
argument-hint: "[message]"
user-invocable: true
model: inherit
effort: medium
hooks:
  preToolExecution:
    - command: "echo pre-hook"
---

# Commit Skill

Review the staged changes and create a commit message following conventional commit format.

1. Run `git diff --staged` to see changes
2. Generate a commit message
3. Run `git commit -m "<message>"` 
```

### Loading Pipeline (1,088 lines in `loadSkillsDir.ts`)

The original loader is **massively over-engineered**:

1. **5 source paths** scanned in parallel (managed, user, project, additional, legacy)
2. **Deduplication** via `realpath()` — resolves symlinks to canonical paths
3. **Conditional skills** — skills with `paths:` frontmatter only activate when matching files are touched
4. **Legacy support** — old `/commands/` directory format alongside new `/skills/` format
5. **MCP skill builders** — creates skills from MCP server resources
6. **Memoized** with `lodash-es/memoize` — computed once per session

### Bundled Skills (16 total)

| Skill | What It Does |
|-------|-------------|
| `batch` | Apply operations across multiple files |
| `claudeApi` | Raw Anthropic API calls |
| `debug` | Systematic debugging workflow |
| `loop` | Iterative refinement until tests pass |
| `remember` | Persist information to CLAUDE.md |
| `simplify` | Simplify complex code |
| `skillify` | Create new skills from current workflow |
| `stuck` | Break out of loops when blocked |
| `verify` / `verifyContent` | Verify code correctness |
| `updateConfig` | Modify configuration files |
| `keybindings` | Configure keyboard shortcuts |
| `scheduleRemoteAgents` | Schedule remote agent execution |
| `loremIpsum` | Generate placeholder text |
| `claudeInChrome` | Chrome extension integration |

### Token Cost of Skills

The skill system injects skill descriptions into the **system prompt** so the model knows what skills are available. Each skill's frontmatter consumes tokens:

```typescript
// From loadSkillsDir.ts
export function estimateSkillFrontmatterTokens(skill: Command): number {
  const frontmatterText = [skill.name, skill.description, skill.whenToUse]
    .filter(Boolean)
    .join(' ')
  return roughTokenCountEstimation(frontmatterText)
}
```

**Estimated cost**: 16 bundled skills × ~50 tokens each = **~800 tokens** per conversation just for skill availability.

---

## 2. Original Agent System

### AgentTool (`src/tools/AgentTool/`)

The `AgentTool` spawns sub-agents — separate instances of the query engine that run independently with their own tool permissions, context, and task scope.

```typescript
// Simplified AgentTool interface
AgentTool({
  description: "Investigate auth module",
  prompt: "Find all null pointer exceptions in src/auth/...",
  subagent_type: "worker",  // or "research", "implementation"
  // model: undefined — inherits from parent
})
```

### Coordinator Mode (`src/coordinator/coordinatorMode.ts`)

When `COORDINATOR_MODE` is enabled, the system prompt transforms the agent into an **orchestrator** that cannot directly use tools but instead manages workers:

**Available tools in coordinator mode:**
- `AgentTool` — Spawn workers
- `SendMessageTool` — Continue/redirect existing workers
- `TaskStopTool` — Kill a misdirected worker

**Worker results arrive as user-role messages** with XML-wrapped task notifications:

```xml
<task-notification>
  <task-id>agent-a1b</task-id>
  <status>completed|failed|killed</status>
  <summary>Agent "Investigate auth bug" completed</summary>
  <result>Found null pointer in src/auth/validate.ts:42...</result>
  <usage>
    <total_tokens>15000</total_tokens>
    <tool_uses>8</tool_uses>
    <duration_ms>4200</duration_ms>
  </usage>
</task-notification>
```

### Coordinator System Prompt (371 lines)

The coordinator system prompt is **massive** — ~4,000 tokens. It includes:
- Role definition (6 sections)
- Tool usage patterns and anti-patterns
- Worker prompt writing guidelines
- Task workflow phases (Research → Synthesis → Implementation → Verification)
- Detailed examples with XML formatting
- Concurrency management rules

---

## 3. Problems with the Original Design

### Token Bloat

| Component | Tokens per Conversation |
|-----------|----------------------:|
| Coordinator system prompt | ~4,000 |
| 16 skill descriptions | ~800 |
| Mode prompt modifications | ~300 |
| Tool schemas (40 tools) | ~2,000 |
| Git context | ~200 |
| CLAUDE.md memory | ~500-5,000 |
| **Minimum baseline** | **~7,800** |

For a 32K context window (typical small model), that's **24% consumed before the user says anything**.

### Over-Abstraction

- 16 separate files for the router
- 7 manager classes with singletons
- 3 different skill loading paths (skills, commands, bundled)
- Complex deduplication via `realpath()`
- Memoization layers that hide state
- Feature flags that gate features but keep the code in the binary

### CPU/Memory Overhead

- React/Ink terminal UI (reconciler running on every keystroke)
- 74 npm dependencies
- GrowthBook feature flag SDK (network requests at startup)
- OpenTelemetry + Sentry telemetry
- Commander.js parsing for 85+ commands

---

## 4. Quantum Code: Redesigned Skill System

### Design Principles

1. **No runtime parsing** — skills are compiled at build time or loaded once
2. **Token budget = capability** — every skill token must earn its place
3. **Frontmatter only** — skill content is lazy-loaded on invocation
4. **No deduplication** — use a single, flat skills directory

### Minimal Skill Format

```markdown
---
name: commit
desc: Git commit with generated message
tools: [bash]
---
Stage check, commit message, commit. Follow conventional commits.
```

**Token cost**: ~15 tokens for frontmatter injection vs ~50 in original.

### Skill Loader (< 100 lines)

```typescript
// skills.ts — complete skill loader
import { readdir, readFile } from 'fs/promises'
import { join } from 'path'

interface Skill {
  name: string
  desc: string
  tools: string[]
  content: string  // lazy-loaded
}

const SKILLS_DIR = '.quantum/skills'
let _skills: Map<string, Skill> | null = null

export async function loadSkills(dir = SKILLS_DIR): Promise<Map<string, Skill>> {
  if (_skills) return _skills
  _skills = new Map()

  try {
    const entries = await readdir(dir, { withFileTypes: true })
    for (const e of entries) {
      if (!e.isDirectory()) continue
      const content = await readFile(join(dir, e.name, 'SKILL.md'), 'utf8')
      const { meta, body } = parseFrontmatter(content)
      _skills.set(meta.name || e.name, {
        name: meta.name || e.name,
        desc: meta.desc || '',
        tools: meta.tools || [],
        content: body,  // stored but not injected into system prompt
      })
    }
  } catch { /* dir doesn't exist — no skills */ }

  return _skills
}

// Inject only names + descriptions into system prompt (minimal tokens)
export function skillManifest(skills: Map<string, Skill>): string {
  if (skills.size === 0) return ''
  const lines = Array.from(skills.values())
    .map(s => `- /${s.name}: ${s.desc}`)
    .join('\n')
  return `Available skills:\n${lines}`
}
```

**Token cost for 10 skills**: ~100 tokens total (vs ~500+ in original).

---

## 5. Quantum Code: Redesigned Agent System

### Problem: Coordinator Prompt is 4K Tokens

The original coordinator prompt contains detailed examples, XML format specs, anti-pattern lists, and workflow phases. For a 7B-13B model, this is untenable.

### Solution: Minimal Coordinator Prompt (~500 tokens)

```
You orchestrate tasks. You direct workers, don't do tool work yourself.

Tools:
- spawn(desc, prompt) — create worker with self-contained instructions
- msg(id, message) — continue existing worker  
- stop(id) — halt worker

Workflow:
1. Research: spawn parallel read-only workers
2. Synthesize: read findings, write specific implementation prompts
3. Implement: spawn worker with exact file paths and changes
4. Verify: spawn fresh worker to test

Rules:
- Worker prompts must be self-contained (they can't see your conversation)
- Include file paths, line numbers, exact changes
- Never write "based on your findings" — synthesize into specifics
- Read-only tasks: parallel. Write tasks: sequential per file.

Worker results appear as: [WORKER id STATUS] result_text
```

### Token Savings

| Component | Original | Quantum Code | Savings |
|-----------|----------|-------------|---------|
| Coordinator prompt | 4,000 | 500 | 87% |
| Worker result format | XML (~100/result) | Plain text (~30/result) | 70% |
| Total per-session | ~5,000 | ~600 | 88% |

---

## 6. Optimized System Prompts for Small Models

### Design Philosophy

Small models (7B-13B, running on CPU/GPU) need **shorter, denser prompts**. Every instruction must be:
- **Actionable** — tells the model what to do, not what it could do
- **Structured** — uses newlines and markers, not prose
- **Measured** — contributes to correct behavior proportional to its token cost

### Original System Prompt Problem

The original mode prompts are verbose:
```
You are in PLAN mode. Your role is to analyze and plan, not to implement.
- Focus on understanding the problem deeply
- Explore the codebase to understand context
- Identify dependencies and risks
- Create structured plans with clear steps
- Estimate scope and complexity
- Suggest implementation approaches
```

That's ~60 tokens for behavioral guidance that could be:
```
MODE: plan (read-only)
Output: Analysis → Approach → Steps → Risks → Dependencies
```

**8 tokens. Same effect.**

### Quantum Code System Prompts

#### Base System Prompt (~200 tokens)

```
You are Quantum, a coding assistant. You work in {cwd}.

Respond concisely. Use tools when needed, not speculatively.

Available tools: {tool_list}

{mode_instruction}
{skill_manifest}
{git_context}
```

#### Mode Instructions (~30 tokens each)

```typescript
const MODE_PROMPTS = {
  chat:   'MODE: chat. Answer directly. Suggest tools only if needed.',
  plan:   'MODE: plan (read-only). Output: Analysis → Steps → Risks.',
  build:  'MODE: build. Make changes. Test. Report progress.',
  review: 'MODE: review (read-only). Output: Issues → Suggestions.',
  debug:  'MODE: debug. Investigate → Root cause → Fix suggestion.',
}
```

#### Tool Schemas (Compressed)

Instead of full Zod schemas per tool (~50 tokens each × 40 tools = 2,000 tokens), use compressed descriptions:

```typescript
const TOOL_SCHEMAS = {
  read:   { params: 'path:string, lines?:range', desc: 'Read file' },
  edit:   { params: 'path:string, old:string, new:string', desc: 'Replace text in file' },
  write:  { params: 'path:string, content:string', desc: 'Create/overwrite file' },
  bash:   { params: 'cmd:string, cwd?:string', desc: 'Run shell command' },
  glob:   { params: 'pattern:string', desc: 'Find files' },
  grep:   { params: 'pattern:string, path?:string', desc: 'Search content' },
  search: { params: 'query:string', desc: 'Web search' },
  ask:    { params: 'question:string', desc: 'Ask user' },
}
```

**8 tools × ~15 tokens = 120 tokens** (vs 2,000 for 40 tools).

### Total System Prompt Budget Comparison

| Component | Original | Quantum Code |
|-----------|----------|-------------|
| Base identity | ~500 | ~50 |
| Mode instructions | ~300 | ~30 |
| Tool schemas | ~2,000 | ~120 |
| Skill manifest | ~800 | ~100 |
| Git context | ~200 | ~100 |
| Memory/CLAUDE.md | ~500-5,000 | ~200 (trimmed) |
| **Total** | **~4,300-8,000** | **~600** |

**At 600 tokens, Quantum Code leaves 31,400 tokens for conversation in a 32K window.**

---

## 7. CPU Optimization Strategies

### Problem: Small models on CPU are slow

A 7B model on an 8-core CPU (no GPU) generates ~5-15 tokens/second. Every wasted token costs 60-200ms of user waiting time.

### Strategies

#### 1. Aggressive Context Trimming

```typescript
// Only inject context that's referenced in the current task
function buildContext(prompt: string, memory: MemoryEntry[]): string {
  const keywords = extractKeywords(prompt)
  return memory
    .filter(m => keywords.some(k => m.content.includes(k)))
    .slice(0, 3)  // Max 3 memory entries
    .map(m => m.content.slice(0, 200))  // Max 200 chars each
    .join('\n')
}
```

#### 2. Speculative Decoding (if supported by runtime)

Use a tiny "draft" model (1B) to generate candidate tokens, verified by the main model. Achieves 2-3x speedup on CPU.

#### 3. KV Cache Persistence

For `llama.cpp` backends, persist the KV cache between turns:
```typescript
// Save KV cache after each turn
await llamaCpp.saveKVCache(sessionId, cacheDir)
// Load on next turn — skips re-processing system prompt
await llamaCpp.loadKVCache(sessionId, cacheDir)
```

This means the ~600-token system prompt is processed **once per session**, not per turn.

#### 4. Quantization-Aware Prompt Design

Quantized models (Q4_K_M, Q5_K_M) have reduced vocabulary precision. Prompts should:
- Use common English words (not jargon)
- Avoid ambiguous phrasing
- Use structured format markers (`MODE:`, `Output:`) that survive quantization
- Keep instructions under 100 tokens

---

## 8. GPU Optimization Strategies

### VRAM Budgeting

| Model Size | VRAM (Q4) | VRAM (FP16) | Max Context |
|-----------|----------|------------|------------|
| 7B | 4GB | 14GB | 8K-32K |
| 13B | 8GB | 26GB | 8K-32K |
| 34B | 20GB | 68GB | 8K-16K |

### Strategies

#### 1. Flash Attention

Enables longer context windows with O(n) memory instead of O(n²):
```bash
# llama.cpp with Flash Attention
./server --flash-attn --ctx-size 32768
```

#### 2. Continuous Batching

If running as a local server, batch multiple tool call results into a single forward pass:
```typescript
// Instead of 3 separate API calls for 3 tool results:
const results = await Promise.all([
  runTool('read', { path: 'a.ts' }),
  runTool('read', { path: 'b.ts' }),
  runTool('grep', { pattern: 'TODO' }),
])
// Combine into single message, single forward pass
const combined = results.map(r => r.content).join('\n---\n')
```

#### 3. GGUF Split for Multi-GPU

For dual-GPU setups (e.g., 2× RTX 3060 12GB):
```bash
./server --model model.gguf --tensor-split 0.5,0.5 --n-gpu-layers 99
```

#### 4. Prompt Caching

Anthropic API supports prompt caching. For local models with `llama.cpp`:
- System prompt is cached after first evaluation
- Only new user input requires full attention computation
- **Effective speedup**: 3-5x for multi-turn conversations

---

## 9. Complete Quantum Code Agent Configuration

### agent.md (Revised)

```markdown
---
name: quantum-agent
---

# Agent Rules

1. Read before write. Understand context first.
2. Small changes. One concern per edit.
3. Match surrounding style. No cosmetic refactors.
4. Validate: typecheck → test → report.
5. When stuck: state what you tried, suggest alternatives.
```

**35 tokens.** Original: ~100 tokens.

### Skill.md (Revised)

The original `Skill.md` (221 lines, 10.7KB) is a **repository skill** — conventions documentation. For Quantum Code, this should be generated from the codebase structure, not hand-maintained.

Replacement: auto-generated at session start:

```typescript
function generateRepoSkill(cwd: string): string {
  const pkg = readPkgJsonSync(cwd)
  const dirs = readdirSync('src').filter(d => d.isDirectory)
  return [
    `Project: ${pkg.name}`,
    `Stack: ${pkg.dependencies ? Object.keys(pkg.dependencies).slice(0, 5).join(', ') : 'unknown'}`,
    `Dirs: ${dirs.join(', ')}`,
    `Scripts: ${Object.keys(pkg.scripts || {}).join(', ')}`,
  ].join('\n')
}
```

**~50 tokens generated dynamically** vs 10.7KB (2,700 tokens) static file.

---

## 10. Token Budget Summary for Quantum Code

### Per-Turn Token Allocation (32K context window)

| Component | Tokens | % of 32K |
|-----------|-------:|:--------:|
| System prompt (base + mode) | 80 | 0.25% |
| Tool schemas | 120 | 0.38% |
| Skill manifest | 100 | 0.31% |
| Git context | 100 | 0.31% |
| Memory (trimmed) | 200 | 0.63% |
| **System total** | **600** | **1.9%** |
| Conversation history | 8,000 | 25.0% |
| Tool results (current turn) | 4,000 | 12.5% |
| Model output budget | 4,000 | 12.5% |
| **Available for user input** | **15,400** | **48.1%** |
| **Buffer** | **0** | **0%** |

### vs Original (200K context window)

| Component | Original Tokens | Quantum Tokens | Reduction |
|-----------|----------------:|---------------:|:---------:|
| System prompt | 4,300 | 80 | **98%** |
| Tool schemas | 2,000 | 120 | **94%** |
| Skill manifest | 800 | 100 | **87%** |
| Context/memory | 5,500 | 300 | **95%** |
| **Total overhead** | **12,600** | **600** | **95%** |
