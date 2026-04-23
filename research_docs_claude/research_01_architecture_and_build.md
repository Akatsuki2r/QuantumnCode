# Research 01 — Architecture, Build System & Execution Pipeline

> Quantum Code codebase deep-dive: how the original Claude Code source is structured, built, and executed — and how to replicate a minimal, optimized variant.

---

## 1. Origin & Source Layout

This codebase is the **leaked Claude Code CLI source** (`@anthropic-ai/claude-code`, v0.0.0-leaked, 2026-03-31). It contains ~1,900 files / 512K+ lines of TypeScript under `src/`, with Rust acceleration modules in `rust/`.

### Root-Level Build Artifacts

| File/Dir | Purpose |
|----------|---------|
| `package.json` | Bun-based monorepo, entry `src/entrypoints/cli.tsx` |
| `bun.lock` / `bunfig.toml` | Bun package manager lock + config |
| `tsconfig.json` | TypeScript strict mode, ES modules |
| `biome.json` | Biome linter/formatter (replaces ESLint+Prettier) |
| `drizzle.config.ts` | Drizzle ORM for SQLite (server mode) |
| `vitest.config.ts` | Vitest test runner |
| `rust/` | Cargo workspace with NAPI-RS bindings |
| `scripts/` | Build scripts (`build-bundle.ts`, `build-web.ts`, `package-npm.ts`) |
| `docker/` + `Dockerfile` | Container deployment |
| `helm/` | Kubernetes Helm charts |
| `grafana/` | Monitoring dashboards |

### Source Directory Map (`src/`)

```
src/
├── main.tsx                 # ~804K — Commander.js CLI parser + React/Ink bootstrap
├── QueryEngine.ts           # ~47K — Core LLM API loop (streaming, tool calls, retries)
├── Tool.ts                  # ~30K — Tool base types, buildTool() factory
├── commands.ts              # ~25K — Command registry (conditional imports by feature flag)
├── context.ts               # ~6K  — System/user context (git status, CLAUDE.md)
├── cost-tracker.ts          # ~11K — Token cost tracking per turn
│
├── router/                  # ★ Quantum Code router (our custom addition)
├── tools/                   # ~40 agent tool implementations
├── commands/                # ~50 slash command implementations
├── skills/                  # Skill system (bundled + user-defined)
├── coordinator/             # Multi-agent orchestration (COORDINATOR_MODE flag)
├── services/                # API client, MCP, OAuth, analytics, plugins
├── bridge/                  # IDE integration (VS Code, JetBrains)
├── components/              # ~140 Ink/React terminal UI components
├── hooks/                   # ~80 React hooks (permissions, input, sessions)
├── state/                   # AppState store (React context + mutable object)
├── entrypoints/             # Initialization: cli.tsx → init.ts → REPL
├── query/                   # Query pipeline (config, deps, stopHooks, token budget)
├── rust/                    # TypeScript bindings to native Rust module
└── [15 more dirs]           # memdir, tasks, vim, voice, schemas, etc.
```

---

## 2. Execution Pipeline (How It Runs)

```
CLI invocation
    │
    ▼
main.tsx (Commander.js parse)
    │ Parallel prefetch: MDM settings, Keychain, GrowthBook feature flags
    ▼
entrypoints/init.ts
    │ Config, telemetry, OAuth, environment detection
    ▼
entrypoints/cli.tsx
    │ CLI session orchestration
    ▼
replLauncher.tsx
    │ React/Ink renderer for terminal UI
    ▼
REPL Screen ←→ QueryEngine.ts ←→ Anthropic API
                    │
                    ▼
              Tool-call loop
              (execute tool → feed result → repeat)
                    │
                    ▼
              Terminal UI (React/Ink components)
```

### Startup Sequence (Critical for Optimization)

1. **`main.tsx`**: Fires `startMdmRawRead()` and `startKeychainPrefetch()` as fire-and-forget promises before heavy module `import()`. This is a **parallel prefetch** pattern — the I/O starts immediately while the module graph evaluates.
2. **`entrypoints/init.ts`**: Config loading, telemetry init, OAuth token refresh, MDM policy application.
3. **`entrypoints/cli.tsx`**: Parses CLI args, initializes React/Ink renderer, launches REPL.
4. **`QueryEngine.ts`**: The 46K-line heart — handles streaming, tool-call loops, thinking mode, retry logic, token counting, and context window management.

### Key Performance Pattern: Lazy Loading

Heavy modules are deferred via dynamic `import()`:
```typescript
// OpenTelemetry (~400KB) only loaded when needed
const otel = await import('@opentelemetry/api')
// gRPC (~700KB) only loaded when needed
const grpc = await import('@grpc/grpc-js')
```

---

## 3. Build System

### Runtime: Bun (not Node.js)

- Native JSX/TSX support without transpilation
- `bun:bundle` feature flags for **dead-code elimination** at build time
- ES modules with `.js` extensions (Bun convention)
- Single-binary output via `bun build --compile`

### Feature Flags (Build-Time DCE)

```typescript
import { feature } from 'bun:bundle'

// Stripped entirely from production builds if flag is off
if (feature('VOICE_MODE')) {
  const voiceCommand = require('./commands/voice/index.js').default
}
```

**Active flags**: `PROACTIVE`, `KAIROS`, `BRIDGE_MODE`, `DAEMON`, `VOICE_MODE`, `AGENT_TRIGGERS`, `MONITOR_TOOL`, `COORDINATOR_MODE`, `WORKFLOW_SCRIPTS`, `BREAK_CACHE_COMMAND`

### Build Commands

```bash
bun scripts/build-bundle.ts          # Development build
bun scripts/build-bundle.ts --minify # Production build
bun scripts/build-web.ts             # Web client build
```

---

## 4. Porting Strategy: What Quantum Code Must Replicate

### Core Pipeline (Must-Have)

| Component | Original | Quantum Code Target |
|-----------|----------|---------------------|
| CLI Parser | Commander.js (25K lines of `commands.ts`) | Slim arg parser (< 500 lines) |
| Query Engine | 46K-line monolith | Modular query pipeline (< 5K lines) |
| Tool System | 40 tools, ~30K types | Core 8-10 tools, lean schemas |
| Router | `src/router/` (12 files, ~150K total bytes) | Keep and optimize |
| Skills | Complex loader (34K lines `loadSkillsDir.ts`) | Frontmatter-only skills (< 2K lines) |
| Context | `context.ts` (git, CLAUDE.md) | Minimal context injector |
| UI | React + Ink (140 components) | Optional — start with plain stdout |

### What to Drop

| Component | Reason |
|-----------|--------|
| Bridge (`src/bridge/`) | IDE integration — not needed for CLI-first |
| Voice (`src/voice/`) | Feature-flagged, niche |
| Buddy (`src/buddy/`) | Easter egg |
| 30+ slash commands | Most are convenience wrappers |
| GrowthBook analytics | A/B testing infra — unnecessary |
| Sentry error tracking | Enterprise overhead |
| Plugin marketplace | Over-engineered for minimalism |
| Remote sessions | Enterprise feature |
| Server mode | Not needed for local-first |

### Estimated Size Reduction

| Metric | Original | Quantum Code Target |
|--------|----------|---------------------|
| Source files | ~1,900 | < 100 |
| Lines of code | 512K+ | < 15K |
| Dependencies | 74 runtime deps | < 15 |
| Binary size | ~50MB | < 5MB |
| Startup time | ~500ms | < 100ms |
| Cold-start to first response | ~2s | < 500ms |

---

## 5. Technology Stack Decisions for Quantum Code

### Keep
- **TypeScript** (strict mode, ES modules)
- **Bun** runtime (or Node.js with esbuild for broader compat)
- **Zod** for schema validation (lightweight, tree-shakeable)
- **Rust** for hot paths (file search, token estimation, prompt analysis)

### Replace
- **React/Ink → plain stdout** (or minimal ANSI library like `picocolors`)
- **Commander.js → custom arg parser** (or `cac` — 5KB vs Commander's 40KB)
- **lodash-es → native ES** (Array.prototype methods, structuredClone)
- **axios → native fetch** (built into Bun/Node 18+)

### New for Quantum Code
- **GGUF model loading** via `llama.cpp` bindings for CPU inference
- **Vulkan/CUDA dispatch** for GPU-accelerated inference
- **ONNX Runtime** as alternative inference backend
- **SQLite** (via `better-sqlite3`) for local state persistence

---

## 6. Build System for Quantum Code

```bash
# Recommended build pipeline
bun build src/main.ts \
  --target=bun \
  --minify \
  --sourcemap=none \
  --compile \
  --outfile=quantum

# Rust native module
cd rust && cargo build --release
# Produces: rust/target/release/libquantum_code_core.so
```

### File Structure Target

```
quantum-code/
├── src/
│   ├── main.ts              # Entry point (< 200 lines)
│   ├── query.ts             # LLM query pipeline (< 2K lines)
│   ├── router/              # Keep existing router (optimized)
│   ├── tools/               # 8-10 core tools
│   ├── skills/              # Minimal skill loader
│   ├── context.ts           # Git + memory context
│   └── config.ts            # Configuration
├── rust/                    # Native acceleration
├── package.json
└── tsconfig.json
```
