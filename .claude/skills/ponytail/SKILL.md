---
name: ponytail
description: Enforce minimal, YAGNI-driven coding practices via a decision ladder before writing any code. Use whenever writing or modifying code in this repo, and especially when the user says "ponytail", "keep it minimal", or asks for a leaner/simpler implementation. Loosely inspired by the public "lazy senior dev" YAGNI-ladder concept; maintained independently in this repo (upstream source not verified — do not pull further content from it).
---

# Ponytail: lazy-senior-dev decision ladder

## Usage
Pass an intensity as the skill argument, e.g. `/ponytail ultra` or `/ponytail lite`. No argument defaults to **full**.

Before writing code, work down this ladder and stop at the first rung that solves the task:

1. **Does it need to exist?** If the task can be skipped or solved without code, skip it.
2. **Already in this codebase?** Reuse existing functions/components/utilities instead of writing new ones.
3. **Covered by stdlib?** Prefer the language/framework standard library over custom code.
4. **Native platform feature?** Prefer built-in platform/browser/DB features over hand-rolled logic.
5. **Existing dependency?** Use a library already in `package.json` / `requirements` / `pom.xml` before adding new code — and before reaching for a *new* dependency, since that's a decision with its own cost (audit, install, blast radius) beyond just code volume.
6. **Can it be one line?** Write it that way.
7. **Otherwise:** ship the minimal working implementation for exactly what was asked.

## Intensity levels
- **lite** — build as requested, but mention the lazier alternative.
- **full** (default) — actively enforce the ladder; stdlib/native/existing-dep first.
- **ultra** — YAGNI extremist; push back on speculative requirements while still shipping a correct minimal solution.

## Non-negotiables
Never simplify away input validation, error handling, security, or accessibility, and never drop something the user explicitly asked for. Read and understand the relevant code fully before deciding where to stop on the ladder — comprehension before laziness.

## Output style
Prefer deletion over addition, boring over clever, fewest files, shortest working diff. If you deliberately stop short of a more robust solution, say so briefly (one line) rather than silently under-building.
