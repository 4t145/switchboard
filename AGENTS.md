# AGENTS.md

## Skeleton UI Reference

For Skeleton UI implementation details, use the official LLM index:

- URL: https://www.skeleton.dev/llms.txt

This index covers documentation for both React and Svelte variants.  
When you need exact component usage, first consult this index, then fetch the target docs page.

### Styling Rule

Always use the **presets** section in Skeleton documentation for style classes.  
Do **not** use legacy `variant-xx` classes unless the existing codebase explicitly depends on them.

---

## Language Policy

### User Communication

Respond in the same language as the user’s latest message:

- If the user writes in Chinese, reply in Chinese.
- If the user writes in English, reply in English.
- If the user switches language, switch accordingly.

### Code and Public Artifacts

Use **English only** for all public-facing engineering content:

- Code comments
- Documentation (README, API docs, design docs)
- Commit messages
- Variable and function names
- Error messages
- Public interfaces and APIs

### Rationale

This policy ensures:

1. Better user interaction in the user’s preferred language
2. Consistent collaboration standards for global teams
3. Maintainable, searchable, and review-friendly code/documentation

---

## Rust Engineering Guidelines (Rust 2024)

### 1) Avoid Magic Numbers

Never hardcode unexplained numeric or string literals.  
Define named `const` values and reference them.

### 2) Document Error Cases

Any function returning `Result` must include a `# Errors` section in its doc comments, describing possible failure cases.

### 3) Prefer Native Async Patterns

Prefer `impl Future` or `BoxFuture` over `async_trait`.  
Do not introduce `async_trait` unless there is a strong, explicit reason.

### 4) Never `unwrap()`

- Restructure control flow to avoid `unwrap()`.
- If unavoidable, use `expect("...reason...")` with a clear and justified message.

### 5) Keep Code Maintainable

- Split long functions into focused helpers.
- Use meaningful names for functions and variables.
- Keep each function single-purpose.
- Add concise comments only for non-obvious logic.

### 6) Flatten Control Flow

Prefer modern Rust patterns that reduce nesting:

- Use `let PATTERN = expr else { ... };` to keep main-path variables in outer scope.
- For non-returning error branches, use patterns like:
  `let Ok(value) = expr.inspect_err(|e| { ... }) else { return; };`
- In `select!`, return a local `Event` enum variant first, then handle it after selection.
- Remember loops can return values; use that when it improves clarity.
- Prefer idiomatic Rust 2024 style consistently.

---
