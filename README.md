# xtop effects

Frame effects and transitions for xtop. Each effect is a visual add-on that
plugs into the kernel's render pipeline through the shared contract crate
`xtop-effect-api` (in the `api` repo): it receives the fully rendered
ratatui buffer of a frame plus the time since the effect started, and
rewrites the buffer in place before the terminal flush.

## Workspace layout

```
effects/
  Cargo.toml           workspace root
  xtop-effect-fade/    Fade: fades a rendered frame in from black (500 ms)
  docs/                effect authoring guide + effect catalog
```

Each effect lives in its own `xtop-effect-<name>/` crate. Shared effect
helpers would live in an `effects-lib/` crate — that crate appears only
once a second effect actually shares code with an existing one, so it is
not created empty.

## Effects

| Crate | Effect | Manifest id | Behavior |
|-------|--------|-------------|----------|
| `xtop-effect-fade` | Fade | `fade` | Fades the rendered frame in from black over 500 ms; deterministic, zero config |

## Documentation

- `docs/write-effect.md` — how to implement an effect against
  `xtop-effect-api` (trait recap, host contract, testing pattern).
- `docs/effects.md` — catalog of the built-in effects and the kernel
  integration note.

## Development

From this repo root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo doc --workspace --no-deps
```

During active development all repos live side by side. The committed form
of `xtop-effect-api` is a floating git dependency
(`{ git = "https://github.com/xtop-cli/api" }`, shared `[workspace.dependencies]`).
To compile against the local `api` checkout instead — required until the
`api` repo is pushed — temporarily point that dependency at
`../api/crates/effect-api` (a path relative to this repo), run the commands
above, then restore the git form and delete the generated `Cargo.lock`
(the repo ignores it by design).

## Local CI

```bash
./scripts/ci.sh            # run every stage
./scripts/ci.sh fmt        # run one stage
```

Stages: `fmt` | `clippy` | `check` | `test`.

## License

MIT
