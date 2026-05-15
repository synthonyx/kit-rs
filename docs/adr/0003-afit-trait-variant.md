# 0003 — `async fn` in trait + `trait_variant` for async dispatch

**Status:** Accepted

## Context

The kit needs async trait methods (`DispatchAsync::call`, lifecycle
`Hooks::on_*`, Phase 2 storage `Backend::read`, etc.). Two options for
async traits in stable Rust:

1. **`#[async_trait]`** (proc-macro). Each async method desugars to
   `fn name(&self, ...) -> BoxFuture<'_, ...>`. Pros: dyn-safe by default;
   familiar. Cons: heap allocation per call; pulls in a proc-macro on every
   async-using crate.

2. **`async fn` in trait (AFIT)**, stable since Rust 1.75. Pros: no heap
   allocation; native compilation. Cons: not directly dyn-safe; the
   returned future has no inherent `Send` bound, so callers using
   `tokio::spawn` need extra plumbing.

We're tokio-first and care about latency in the dispatch path. The kit
also wants to support `dyn`-erased dispatchers (e.g., a runtime composer
that holds `Box<dyn DispatchAsync<T>>`).

## Decision

Use **AFIT** for all async trait methods, and apply
`#[trait_variant::make(NameSend: Send)]` to generate a `Send`-bounded
variant alongside the local-future variant. Concretely:

```rust
#[trait_variant::make(DispatchAsyncSend: Send)]
pub trait DispatchAsync<T: Config> {
    type Output;
    async fn call(&self, origin: T::Origin) -> Result<Self::Output, DispatchError>;
}
```

Callers that need `Send` futures bound on `DispatchAsyncSend`; callers
that don't (single-threaded executors, RTM-internal code) use the bare
`DispatchAsync`. The `trait-variant` crate is a tiny dep (no runtime cost).

For `Hooks` we use plain AFIT without the Send variant, because hook
invocation is concrete-type from the runtime composer and Send-ness flows
through naturally from the implementing type.

## Consequences

**Positive.** No heap allocation per dispatch. No `async_trait` proc-macro
dependency cascade through every crate. Future Rust improvements to AFIT
(e.g. better object-safety) we get automatically.

**Negative.** Two trait variants per async trait means a slight learning
curve. The MSRV is effectively bound at 1.85 (we'd have wanted ≥1.75
already for AFIT; 1.85 is forced by edition 2024 anyway).

**Forward path.** If Rust eventually stabilises object-safe AFIT directly,
`trait_variant` becomes obsolete and can be removed without API breakage.
