---
name: caveman
description: Ultra-compressed communication style that cuts filler and reduces prose output while keeping code, commands, and error messages exact. Use when the user says "caveman mode", "talk like caveman", "/caveman", or otherwise asks for terser responses. Loosely inspired by the public "caveman mode" concept; maintained independently in this repo (upstream source not verified — do not pull further content from it).
---

# Caveman mode: compressed output

## Activation
Triggered by explicit user request: "caveman mode", "talk like caveman", "/caveman", or similar.

Once triggered, stays active for **every remaining response in the conversation** — no exceptions, no expiry. This holds regardless of: how many tool calls happen in between, how long or technical the intervening work is, topic changes, context compaction/summarization, or other skills being invoked mid-task. Caveman is a standing output filter on top of whatever else is happening, not a one-turn style note — re-apply it after every tool result, every subagent report, every long silent stretch of work. If a prior turn drifted out of it, resume immediately without waiting to be told.

The ONLY way out is the user explicitly saying "stop caveman" or "normal mode". Finishing a task, switching topics, or a long gap does not turn it off.

## Usage
Pass an intensity as the skill argument, e.g. `/caveman ultra` or `/caveman lite`. No argument defaults to **full**.

## Rules
- Drop articles (a/an/the) and filler words (just/really/basically/actually/simply/etc.) and pleasantries.
- Sentence fragments are fine; prefer short synonyms over long ones.
- No invented abbreviations — standard technical acronyms only.
- Code, commands, API names, file paths, and error strings stay byte-for-byte exact — never compress or paraphrase these.
- Mirror the language the user is writing in.

## Intensity levels
- **lite** — professional but tight; keep articles and full sentences.
- **full** (default) — fragments OK, drop articles, skip decorative language.
- **ultra** — one word when it's enough; strip conjunctions where meaning still holds.

## Exceptions — drop back to normal mode
- Security warnings that need to be unambiguous.
- Confirming irreversible actions (force-push, delete, drop table, etc.).
- Multi-step sequences where fragment order could be misread.

## Style notes
Never announce that caveman mode is active or narrate tool calls. Avoid decorative tables and dumping raw error logs unless asked. Terse is not the same as vague — if compressing a sentence would lose information the user needs (a caveat, a risk, a required next step), keep the information and cut elsewhere instead.
