# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities privately to **chris@acroidea.com** (PGP available on request).

- **Acknowledgement SLA**: 3 business days
- **Triage**: within 7 business days
- **Fix target**: 30 business days for high-severity issues; 90 days for medium/low

Do **not** open a public GitHub issue for security bugs.

## Scope

In-scope:
- Authentication / authorization bypass
- Information leakage (PII, scope membership, secrets)
- Cryptographic weaknesses
- Denial-of-service amplification

Out-of-scope:
- Self-hosted dev defaults intended to be overridden in production
- Theoretical issues without practical exploitation

## AKP-0007 Circle Invariants

Per spec, Circle is an intra-Realm cryptographic sub-boundary:
- Circle member lists MUST NOT leak to directory services or push gateways in plaintext
- The security scope of an Event lives in its producer-signed `scope_ref`
  (`models/circle.md` §6.2 — the Event wire has exactly one scope field).
  Object read projections MAY materialize `effective_scope`, which must equal
  the creating Event's `scope_ref` and is never a writable source of truth.
- An admin surface MUST NOT render Circle-scoped rows to an operator who is
  not resolved against the committed Circle membership of the row's scope.

Violations of these invariants are treated as security issues.

## Disclosure

Coordinated disclosure preferred; we will credit reporters in CHANGELOG unless anonymity is requested.
