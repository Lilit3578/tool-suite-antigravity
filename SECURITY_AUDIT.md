# 🔒 Security Gap Analysis Report

**Audit Date**: 2025-12-17  
**Auditor**: Principal Security & Systems Auditor  
**Target**: `tool-suite-antigravity` monorepo (Tauri + Next.js)

---

## Executive Summary

| Status | Count |
|--------|-------|
| ✅ Verified | 4 |
| ⚠️ RISK | 3 |
| ❌ CRITICAL | 2 |

**Production Readiness: 🔴 NOT READY** — Critical authentication vulnerability and missing defense-in-depth controls.

---

## Audit Results

### 1. React Version Consistency

| Check | Result |
|-------|--------|
| `apps/desktop/package.json` | `"react": "^19.1.0"` |
| `apps/web/package.json` | `"react": "^19.1.0"` |

**⚠️ RISK**: Both apps use React 19, which is consistent but **React 19 is still experimental**. Next.js 14 officially supports React 18.x.

> **Issue**: React 19 may introduce breaking changes. Shared UI components between apps could break unexpectedly.

**Evidence**:
```json
// apps/desktop/package.json:29
"react": "^19.1.0"

// apps/web/package.json:18
"react": "^19.1.0"
```

---

### 2. Type Safety (Rust → TypeScript)

| Check | Result |
|-------|--------|
| `specta` in Cargo.toml | ❌ Not found |
| `rspc` in Cargo.toml | ❌ Not found |
| `ts-rs` in Cargo.toml | ✅ Present (`ts-rs = "10.0"`) |
| Manual types.ts | ⚠️ Present (121 lines) |

**⚠️ RISK**: Using manual type definitions in `src/types.ts`. `ts-rs` is present but appears unused for automatic generation.

> **Issue**: Manual types drift from Rust structs → runtime crashes when frontend expects different shapes.

**Evidence**:
```rust
// Cargo.toml:51
ts-rs = "10.0"
```
```typescript
// src/types.ts:1
// Type definitions matching Rust backend types (MANUALLY MAINTAINED)
```

---

### 3. Authentication Security (PKCE)

| Check | Result |
|-------|--------|
| Code verifier generation in Rust | ❌ Not found |
| Token exchange flow | ❌ Not implemented |
| Direct token in deep link | ❌ **CRITICAL VULNERABILITY** |

**❌ CRITICAL**: Deep links pass tokens directly via `prodwidgets://?token=...`

> **This is a credential theft vector**. Any app can register the same URL scheme and intercept tokens.

**Evidence**:
```rust
// lib.rs:41-44
// 1. Parse Token (Legacy/Direct Token)
if let Some(token_pair) = url.query_pairs().find(|(key, _)| key == "token") {
    let token = token_pair.1.to_string();
    println!("Deep Link Token Found: {}", token);  // TOKEN LOGGED IN PLAINTEXT
    // ... existing token logic ...
    let _ = handle_clone.emit("auth-deep-link", token);
}
```

**Required Fix**: Implement PKCE flow:
1. Desktop generates `code_verifier` (random 43-128 char string)
2. Desktop opens web auth with `code_challenge = SHA256(code_verifier)`
3. Web redirects back with `otp` only
4. Desktop exchanges `otp + code_verifier` for token via backend

---

### 4. Rate Limiting (Defense in Depth)

#### Web App (`apps/web/middleware.ts`)

| Check | Result |
|-------|--------|
| `@upstash/ratelimit` in middleware | ❌ Not found |
| Route protection | ✅ Auth routes protected |

**⚠️ RISK**: No rate limiting on API endpoints. Vulnerable to:
- Brute force attacks on auth endpoints
- Resource exhaustion via repeated API calls

**Evidence**:
```typescript
// middleware.ts - NO rate limiting imports
import NextAuth from "next-auth"
import authConfig from "./auth.config"
import { NextResponse } from "next/server"
```

#### Desktop App (`apps/desktop/src-tauri`)

| Check | Result |
|-------|--------|
| `governor` crate | ❌ Not found in Cargo.toml |
| Command rate limiting | ❌ None implemented |

**Evidence**:
```toml
# Cargo.toml - governor NOT present
# Dependencies do NOT include rate limiting crate
```

#### External API Calls

| Service | Rate Limited | Queued |
|---------|--------------|--------|
| Translation API | ❌ No | ❌ No |
| Currency API | ❌ No | ❌ No |

> **Risk**: Unbounded external API calls can exhaust API quotas and cause service outages.

---

### 5. Clipboard Safety

| Check | Result |
|-------|--------|
| Passive listener (change count) | ✅ Implemented |
| `enigo` crate | ✅ Not used |
| Simulated Cmd+C | ⚠️ Present (fallback only) |
| Sensitive content filter | ✅ Implemented |

**✅ Verified**: Clipboard monitoring uses safe passive polling.

**Evidence**:
```rust
// monitor.rs:62 - Uses tauri-plugin-clipboard-manager
let sleep_interval = match app.clipboard().read_text() {
    Ok(current_content) => { ... }
}

// filter.rs:12-17 - Sensitive content detection
Regex::new(r"ghp_[a-zA-Z0-9]{36}").expect("Invalid GitHub token regex"),
Regex::new(r"xox[baprs]-[a-zA-Z0-9]{10,48}").expect("Invalid Slack token regex"),
```

**Note**: `capture_via_simulated_copy()` in `macos.rs` uses CGEvent to simulate Cmd+C as a **fallback** when Accessibility API fails. This is acceptable but should be documented.

---

### 6. Permission Handling

| Check | Result |
|-------|--------|
| `check_accessibility_permissions()` exists | ✅ Yes |
| Called during `main.rs` / app setup | ❌ No |
| Called before each sensitive operation | ✅ Yes |

**✅ Verified with recommendation**: Permission check exists but is reactive (per-operation), not proactive (app startup).

**Evidence**:
```rust
// macos.rs:26 - Function exists
pub fn check_accessibility_permissions() -> bool {
    unsafe { AXIsProcessTrusted() }
}

// macos.rs:78, 106, 176... - Called before operations
if !check_accessibility_permissions() {
    return Err(AppError::System("Accessibility permissions denied".to_string()));
}
```

**Recommendation**: Add proactive check in `lib.rs` setup to prompt user immediately on first launch.

---

## Summary Table

| Standard | Status | Severity |
|----------|--------|----------|
| React 18.x Consistency | ⚠️ RISK | Medium |
| Type Safety (specta/rspc) | ⚠️ RISK | Medium |
| PKCE Authentication | ❌ CRITICAL | Critical |
| Web Rate Limiting | ❌ MISSING | High |
| Desktop Rate Limiting | ❌ MISSING | High |
| Clipboard Passive Listener | ✅ Verified | — |
| Sensitive Content Filter | ✅ Verified | — |
| Permission Check Exists | ✅ Verified | — |
| Startup Permission Prompt | ⚠️ Improvement | Low |

---

## Action Plan

### Critical (Block Production Launch)

| # | File | Action |
|---|------|--------|
| 1 | `apps/web/lib/actions/auth.ts` | Create PKCE server action: generate `code_challenge`, verify `code_verifier` |
| 2 | `src-tauri/src/commands/auth.rs` | Generate `code_verifier`, store in keychain, send `code_challenge` to web |
| 3 | `src-tauri/src/lib.rs:41-53` | Remove direct token handling; implement OTP → token exchange with verifier |
| 4 | `apps/web/middleware.ts` | Add `@upstash/ratelimit` for auth routes (10 req/min per IP) |

### High Priority (Before Beta)

| # | File | Action |
|---|------|--------|
| 5 | `Cargo.toml` | Add `governor = "0.6"` for command rate limiting |
| 6 | `src-tauri/src/api/commands/*.rs` | Wrap external API calls with governor rate limiter |
| 7 | `src/types.ts` | Replace with auto-generated types from `ts-rs` or migrate to `specta` |

### Medium Priority (Before GA)

| # | File | Action |
|---|------|--------|
| 8 | `apps/*/package.json` | Downgrade to `react@^18.2.0` for Next.js 14 compatibility |
| 9 | `src-tauri/src/lib.rs` | Add proactive `ensure_accessibility_permissions()` call in setup |

---

## Verification Plan

After implementing fixes, verify with:

```bash
# 1. Type safety - ensure types compile
cd apps/desktop && npm run type-check

# 2. Web middleware - test rate limiting
curl -X POST http://localhost:3000/api/auth/signin -d '{"email":"test@test.com"}' \
  && for i in {1..15}; do curl -s -o /dev/null -w "%{http_code}\n" -X POST http://localhost:3000/api/auth/signin -d '{"email":"test@test.com"}'; done
# Expect: 429 after 10 requests

# 3. PKCE verification
# Manual test: Deep link should NOT contain token, only OTP
# Intercept with: open -a "Console" && log stream --predicate 'process == "productivity-widgets"'
```
