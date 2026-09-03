# xtop effects

Xtop effects to animate and enhance your TUI.

Effects are visual/behavioral add-ons (intro animations, transitions, frame
post-processing, ...) that plug into the kernel's render pipeline through
`xtop-effect-api`.

## Workspace

Each effect lives in its own folder under `effects/`:

```
effects/
  effects-lib/         shared helpers for effects
  xtop-effect-<name>/
    Cargo.toml
    src/
    README.md
```

## Getting started (development)

From this repo root:

```bash
cargo build --workspace
```

During active development all repos live side by side and use local path
dependencies:

```
xtop/           kernel
api/            API crates
plugins/
effects/        this repo
extensions/
```

## License

MIT
