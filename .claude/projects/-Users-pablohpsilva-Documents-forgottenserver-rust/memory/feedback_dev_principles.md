---
name: Development principles
description: Core coding principles the user always wants applied: TDD-first, SOLID, DRY, KISS
type: feedback
---

Always apply these four principles when writing or reviewing code:

- **TDD-first**: Write the failing test before any implementation. No exceptions — this also aligns with the mandatory TDD rule in CLAUDE.md.
- **SOLID**: Single responsibility, open/closed, Liskov substitution, interface segregation, dependency inversion. Applies to struct/trait design in Rust.
- **DRY**: Don't repeat yourself. Extract shared logic rather than duplicating it across call sites.
- **KISS**: Keep it simple. Prefer the simplest correct solution; avoid over-engineering.

**Why:** User explicitly requested these be applied consistently to all code in this project.

**How to apply:** Before implementing anything, ask: does the test exist? Is the abstraction the simplest one that fits? Is any logic duplicated? Does each type/function have one clear responsibility?
