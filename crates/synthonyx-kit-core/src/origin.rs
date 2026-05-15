//! Origin types — who triggered a dispatch.

use crate::correlation::CorrelationId;

/// The kind of origin — a low-cardinality tag used by audit log entries and
/// authorization decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OriginKind {
    /// Internal trigger (boot, scheduler, hooks).
    System,
    /// Service-to-service call (sidecar or peer).
    Service,
    /// Authenticated end-user.
    User,
    /// Pre-authentication caller (e.g. login attempt).
    Anonymous,
}

/// The contract every dispatch origin must satisfy.
///
/// Carries enough information for audit (`kind`, `correlation_id`) and
/// authorization (`principal`). RTMs can use [`BaseOrigin`] or define their
/// own origin type implementing this trait.
pub trait OriginTrait: Clone + Send + Sync + 'static {
    /// The principal type (account, service identity, etc.).
    type Principal: core::fmt::Debug + Clone + Send + Sync + 'static;

    /// The authenticated principal, if any.
    fn principal(&self) -> Option<&Self::Principal>;

    /// The kind of origin, for audit/authz dispatch.
    fn kind(&self) -> OriginKind;

    /// The correlation id propagated with this dispatch.
    fn correlation_id(&self) -> CorrelationId;
}

/// A generic origin enum sufficient for most RTMs.
///
/// RTMs with richer origin requirements may define their own type
/// implementing [`OriginTrait`] instead of using this enum.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BaseOrigin<P>
where
    P: core::fmt::Debug + Clone + Send + Sync + 'static,
{
    /// Internal trigger with no external principal.
    System {
        /// Correlation id for this internal flow.
        correlation: CorrelationId,
    },
    /// Service-to-service call.
    Service {
        /// Calling service name.
        name: &'static str,
        /// Service principal (typically a service identity / SPIFFE id).
        principal: P,
        /// Inbound correlation id.
        correlation: CorrelationId,
    },
    /// Authenticated end-user.
    User {
        /// User principal.
        principal: P,
        /// Correlation id.
        correlation: CorrelationId,
    },
    /// Pre-authentication caller.
    Anonymous {
        /// Correlation id.
        correlation: CorrelationId,
    },
}

impl<P> OriginTrait for BaseOrigin<P>
where
    P: core::fmt::Debug + Clone + Send + Sync + 'static,
{
    type Principal = P;

    fn principal(&self) -> Option<&P> {
        match self {
            Self::System { .. } | Self::Anonymous { .. } => None,
            Self::Service { principal, .. } | Self::User { principal, .. } => Some(principal),
        }
    }

    fn kind(&self) -> OriginKind {
        match self {
            Self::System { .. } => OriginKind::System,
            Self::Service { .. } => OriginKind::Service,
            Self::User { .. } => OriginKind::User,
            Self::Anonymous { .. } => OriginKind::Anonymous,
        }
    }

    fn correlation_id(&self) -> CorrelationId {
        match self {
            Self::System { correlation }
            | Self::Service { correlation, .. }
            | Self::User { correlation, .. }
            | Self::Anonymous { correlation } => *correlation,
        }
    }
}
