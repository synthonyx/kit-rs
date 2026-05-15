//! Trace and span identifiers.

use core::fmt;

/// A W3C-compatible trace identifier (16 bytes, 128 bits).
///
/// Carried on every dispatch (via [`crate::OriginTrait::correlation_id`]) and
/// recorded on every audit entry, so a single logical operation can be
/// stitched across logs, audit records, and downstream service calls.
/// Generation strategies (random, derived, extracted from headers) live in
/// `synthonyx-kit-tracing`.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CorrelationId(pub [u8; 16]);

impl CorrelationId {
    /// The zero correlation id, used for system-internal operations that have
    /// no caller context (boot, periodic sweeps, etc.).
    pub const ZERO: Self = Self([0u8; 16]);

    /// True if this is [`Self::ZERO`].
    pub const fn is_zero(&self) -> bool {
        let mut i = 0;
        while i < 16 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl fmt::Debug for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CorrelationId({self})")
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

/// A W3C-compatible span identifier (8 bytes, 64 bits).
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpanId(pub [u8; 8]);

impl SpanId {
    /// The zero span id.
    pub const ZERO: Self = Self([0u8; 8]);
}

impl fmt::Debug for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpanId({self})")
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_id_display_is_lowercase_hex() {
        let id = CorrelationId([
            0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82, 0x93, 0xa4, 0xb5, 0xc6, 0xd7,
            0xe8, 0xf9,
        ]);
        assert_eq!(id.to_string(), "0a1b2c3d4e5f60718293a4b5c6d7e8f9");
    }

    #[test]
    fn zero_is_zero() {
        assert!(CorrelationId::ZERO.is_zero());
        assert!(!CorrelationId([1u8; 16]).is_zero());
    }
}
