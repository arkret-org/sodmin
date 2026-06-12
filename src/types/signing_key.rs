//! DTO shapes for the NotaryWorker signing-key admin surface.
//!
//! These mirror the join-projection of the principal-server's notary
//! signing-key configuration that soland exposes via the admin describe
//! endpoint.

use serde::{Deserialize, Serialize};

/// Origin of the NotaryWorker signing key. `Configured` means an
/// operator-provisioned PEM is loaded from disk / KMS; `Ephemeral` means
/// the worker generated a key in-memory at startup (insecure for
/// production — the operator should rotate to a configured key ASAP).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningKeyOrigin {
    Configured,
    Ephemeral,
}

impl SigningKeyOrigin {
    pub fn label(&self) -> &'static str {
        match self {
            SigningKeyOrigin::Configured => "Configured",
            SigningKeyOrigin::Ephemeral => "Ephemeral",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "configured" => Some(SigningKeyOrigin::Configured),
            "ephemeral" => Some(SigningKeyOrigin::Ephemeral),
            _ => None,
        }
    }
}

/// Read-only describe view for the current NotaryWorker signing key.
/// soland projects this from the running worker's bound key material; the
/// `verification_method_id` is the canonical `<did>#<kid>` reference admins
/// can search for in audit logs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SigningKeyDescribe {
    /// Wire-format `SigningKeyOrigin` (snake_case).
    pub origin: String,
    /// `did:...#kid` of the verification method the worker uses for Seals
    /// with.
    pub verification_method_id: String,
    #[serde(default)]
    pub did: Option<String>,
    #[serde(default)]
    pub kid: Option<String>,
    #[serde(default)]
    pub algorithm: Option<String>,
    /// ISO-8601 last rotation timestamp, when known.
    #[serde(default)]
    pub last_rotated_at: Option<String>,
    /// Whether the operator can trigger a rotation. Ephemeral keys are
    /// never rotatable from the UI — the operator must redeploy with a
    /// configured key first.
    #[serde(default)]
    pub rotatable: bool,
}

impl SigningKeyDescribe {
    /// Typed origin, falling back to `Ephemeral` for unknown wire values
    /// so the UI surfaces the safer (more alarming) state by default.
    pub fn origin_typed(&self) -> SigningKeyOrigin {
        SigningKeyOrigin::from_wire(&self.origin).unwrap_or(SigningKeyOrigin::Ephemeral)
    }

    /// Whether the `Rotate signing key` button should be enabled. Ephemeral
    /// keys cannot be rotated in-place — the operator MUST redeploy with a
    /// configured key first; otherwise rotation would just spin up another
    /// ephemeral key, leaving notary signatures without stable DID binding.
    pub fn can_rotate(&self) -> bool {
        self.rotatable && matches!(self.origin_typed(), SigningKeyOrigin::Configured)
    }
}

/// Outcome shape returned by the rotate endpoint. soland generates a
/// fresh key, swaps the worker's signer atomically, and reports the new
/// verification method id.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotateSigningKeyOutcome {
    pub verification_method_id: String,
    #[serde(default)]
    pub rotated_at: Option<String>,
    #[serde(default)]
    pub previous_verification_method_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_round_trips_through_wire() {
        for (wire, expected) in [
            ("configured", SigningKeyOrigin::Configured),
            ("ephemeral", SigningKeyOrigin::Ephemeral),
        ] {
            let s = SigningKeyOrigin::from_wire(wire).expect("known wire variant");
            assert_eq!(s, expected);
        }
        assert!(SigningKeyOrigin::from_wire("CONFIGURED").is_none());
        assert!(SigningKeyOrigin::from_wire("garbage").is_none());
    }

    #[test]
    fn origin_typed_defaults_to_ephemeral_on_unknown() {
        // Default to the safer (more-alarming) state so unknown wire data
        // doesn't silently look like a configured key.
        let d = SigningKeyDescribe {
            origin: "mystery".into(),
            ..Default::default()
        };
        assert_eq!(d.origin_typed(), SigningKeyOrigin::Ephemeral);
    }

    #[test]
    fn can_rotate_requires_configured_origin_and_rotatable_flag() {
        let configured_rotatable = SigningKeyDescribe {
            origin: "configured".into(),
            rotatable: true,
            ..Default::default()
        };
        assert!(configured_rotatable.can_rotate());

        let configured_locked = SigningKeyDescribe {
            origin: "configured".into(),
            rotatable: false,
            ..Default::default()
        };
        assert!(!configured_locked.can_rotate());

        let ephemeral_even_if_rotatable = SigningKeyDescribe {
            origin: "ephemeral".into(),
            rotatable: true,
            ..Default::default()
        };
        // Ephemeral keys are never rotatable from the UI — the operator
        // must redeploy with a configured key first.
        assert!(!ephemeral_even_if_rotatable.can_rotate());
    }

    #[test]
    fn origin_label_is_capitalized_for_display() {
        assert_eq!(SigningKeyOrigin::Configured.label(), "Configured");
        assert_eq!(SigningKeyOrigin::Ephemeral.label(), "Ephemeral");
    }
}
