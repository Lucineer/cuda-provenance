# cuda-provenance

Provenance — decision lineage, data origin tracing, accountability chains, audit trails (Rust)

Part of the Cocapn fleet — a Lucineer vessel component.

## What It Does

### Key Types

- `DecisionRecord` — core data structure
- `LineageEntry` — core data structure
- `AccountabilityEntry` — core data structure
- `AuditEvent` — core data structure
- `ProvenanceQuery` — core data structure
- `ProvenanceTracker` — core data structure

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-provenance.git
cd cuda-provenance

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_provenance::*;

// See src/lib.rs for full API
// 10 unit tests included
```

### Available Implementations

- `ProvenanceTracker` — see source for methods

## Testing

```bash
cargo test
```

10 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: other
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates


## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
