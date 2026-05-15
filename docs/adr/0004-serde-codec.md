# 0004 — Serde-based `Codec` trait, bincode2 default

**Status:** Accepted

## Context

Storage backends need a way to encode RTM values for persistence and decode
them back. Options considered:

1. **`serde`** abstract trait. Pros: maximum ecosystem coverage (almost
   every type has serde impls); easy to plug in different wire codecs
   (bincode, MessagePack, CBOR). Cons: not byte-stable across serde
   versions if not carefully managed.

2. **`prost`** (protobuf). Pros: schema-first; great for cross-language
   sidecars and gRPC. Cons: every type must be a protobuf message; enums
   map awkwardly.

3. **`bincode`** as a concrete codec. Pros: fast, compact, serde-compatible.
   Cons: locks the wire format at the kernel level.

## Decision

The kernel exposes a `Codec` trait in `synthonyx-kit-storage` with
`encode` / `decode` methods. The trait is intentionally abstract — backends
in Phase 2 choose the concrete codec.

Phase 2's default storage codec is `bincode2` (the 2.x line), applied via a
blanket impl on top of `serde::Serialize + serde::de::DeserializeOwned`.
RTMs that need a specific wire format (e.g. protobuf for gRPC payloads)
override `Codec` directly for their domain types.

## Consequences

**Positive.** Maximum ecosystem leverage — anyone writing an RTM gets
serde derives for free. RTM authors can opt into prost on a per-type basis
without touching the kernel. The kernel doesn't carry a wire format
commitment.

**Negative.** Cross-version stability of serde encodings is the backend's
responsibility, not the kernel's. Switching codecs in production requires
a migration window because the on-disk bytes are not interchangeable.

**Forward path.** Additional codec adapters (CBOR, MessagePack, or
domain-specific schemas) can be added in Phase 2 as thin wrappers without
changing the `Codec` trait shape.
