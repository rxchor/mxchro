# Mxchro documentation

Read only the guides relevant to the task; there is no need to load this entire
directory. Code paths in these guides are relative to the repository root.

| You need to… | Document |
| --- | --- |
| Run, build, debug or add tests | [Development](development.md) |
| Generate the NSIS installer or change the EXE icon | [Packaging](packaging.md) |
| Use macros, the editor, capture and collections | [Usage](usage.md) |
| Understand layers, DTOs, commands and where to make changes | [Architecture](architecture.md) |
| Change components, forms, notifications or dragging | [UI](ui.md) |
| Change hooks, hotkeys, simulation or execution | [Engine and input](engine.md) |
| Change paths, PoloDB, transactions, schema or reset | [Data and persistence](data.md) |
| Change themes, logos, preferences or language | [Appearance and language](appearance.md) |
| Review limitations and previous checks | [Validation](validation.md) |
| Find test suite structure and commands | [Tests](../tests/README.md) |
| Run graphical fixtures or measure drawing CPU time | [Native checks](../tests/native/README.md) · [Profiling](../tests/perf/README.md) |
| Review implemented tasks, pending work and ideas | [TODO](../TODO.md) |

## Maintaining these guides

- `README.md`: introduction, quick start and links, not technical history.
- Local, Git-ignored `AGENTS.md`: essential constraints and task-specific reading.
  Public documentation must not depend on this file being present in a clone.
- `docs/`: behavior and area-specific rules, without repeating them in every index.
- `TODO.md`: migration status and pending work; implemented does not mean
  physically validated or ready to distribute.
- `tests/`: testing instructions and methods alongside their fixtures.

Keep project documentation, READMEs, explanatory comments and examples in English.
Preserve identifiers, mapper names, persisted values and user-provided content.
Update the relevant guide when behavior changes. Do not turn a local or historical
result into a general guarantee or claim a check that has not been performed.
