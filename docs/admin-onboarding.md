# sodmin Admin Onboarding

This guide walks a new Contrix server administrator from a clean
deployment to a working sodmin session with the right capabilities to
do real work. The flow is:

1. **DID setup** — the admin's own controller DID is bound to a coauth account.
2. **SSO config** — coauth knows the upstream identity provider so the admin can sign in.
3. **First login** — the admin opens sodmin in a browser and completes the OAuth2 PKCE flow.
4. **Capability grant flow** — coauth issues the `cx.*` admin scopes the admin needs to drive sodmin.

P5 — until an admin has both a bound DID **and** the right capability
grants, sodmin will surface every destructive action with a "not in
your grant list" hint via the `GrantedCapabilitiesView` component,
and the backend will reject any forged request with
`cx.error.capability_denied`.

## 1. Admin DID Setup

Each administrator is identified by a Decentralized Identifier (DID)
under the round-4 grammar `^did:[a-z0-9]+:[^\s]+$`. Method segments
MUST be lowercase ASCII alphanumeric — no `.`/`-`/`_`/`:`.

Examples that pass the SDK normalizer:

* `did:web:admin.example.org`
* `did:key:z6MkXYZ`
* `did:webvh:auth.example.net:admin-1`

Pick one DID method appropriate for your deployment (`did:web` is the
typical choice for a self-hosted server). The admin generates the
keypair on a secure host and publishes the DID document at the
location implied by the method.

<!-- TODO(screenshot): /agents/personal/new — provision wizard step 1 showing the controller DID field with inline ValidatedInput error -->

Once the DID is published, bind it to the admin's coauth account via
the **Account Detail → Managed DID Bindings → Add binding** form on
`/coauth/accounts/:account_id`. The `control_proof` field is a
base64url-encoded signature challenge — the exact shape depends on
the DID method; see the coauth admin handler docs for the per-method
contract.

## 2. SSO Configuration

sodmin itself does not perform authentication: it redirects every
unauthenticated user to coauth, which handles the OAuth2 PKCE
authorization code flow against the configured upstream provider.

Configure the upstream provider on coauth via **Upstream Providers**
(`/coauth/upstream-providers`):

1. Click **Add provider**.
2. Pick the kind — typically `oidc` for generic OIDC IdPs.
3. Fill in the issuer URL, client id, and client secret.
4. Save.

coauth will dynamically register the redirect URI back to sodmin's
`/oauth/callback` route. Verify the provider appears in
`/coauth/connector-health` as **Healthy** before asking the new admin
to log in — a degraded connector will surface a confusing error on
first sign-in.

<!-- TODO(screenshot): /coauth/upstream-providers — list view with one Healthy upstream provider -->

## 3. First Login

The admin opens the sodmin URL in a browser. The flow is:

1. sodmin SPA loads, reads `/config.json`, learns the `coauth_public_url`.
2. SPA detects no session and redirects to `/login`.
3. `/login` page renders **Sign in with SSO** — clicking it kicks off the OAuth2 PKCE flow against coauth.
4. coauth redirects to the upstream IdP, which authenticates the user and redirects back to coauth.
5. coauth mints an authorization code, redirects to sodmin's `/oauth/callback`.
6. sodmin exchanges the code for an httpOnly session cookie via coauth.
7. SPA navigates to `/dashboard`.

If the admin's account is not marked `is_admin: true` on coauth, the
SPA renders **This account is not a server administrator** and
refuses to load the admin surface. Mark the account admin via the
**Coauth Accounts** page or the coauth admin CLI before retrying.

<!-- TODO(screenshot): /login — sign-in landing page with the SSO button and the auth status panel -->

## 4. Capability Grant Flow

Being signed in as an admin does NOT, by itself, grant the right to
drive every admin operation. The Contrix model is capability-based:
each admin action is gated by a specific `cx.*` capability scope,
and the operator must hold that scope (or a covering parent scope)
before the backend will accept the request.

For sodmin administration the recommended starter grant set is:

| Capability | Why |
| --- | --- |
| `cx.agent.manage` | Meta-scope for the 11 personal-agent admin endpoints. |
| `cx.circle.create` / `cx.circle.manage` | Create and administer Circles (P3A.4). |
| `cx.realm.admin` | Destroy / classify Realms. |
| `cx.audit.read` | Read the audit log on `/audit`. |

Grant capabilities via **Capabilities** (`/capabilities`) — click
**Grant capability**, fill in the grantee DID (the admin's bound
DID, not the coauth `sub`), pick the scope, and submit. The grant
is published as a Move and becomes visible to the admin on their
next page load.

<!-- TODO(screenshot): /capabilities — grant capability dialog with the round-4 ValidatedInput on the grantee DID field -->

Verify the grants landed on the admin's account by opening any
destructive form (e.g. **Provision new agent** on `/agents/personal`).
The `GrantedCapabilitiesView` panel at the top of the wizard will
list the operator's current grants and highlight whether the action's
required capability is present. If not, the **Submit** click will
fail with `cx.error.capability_denied` — request the missing grant
from a higher-privileged admin or via the coauth bootstrap migration.

## Troubleshooting

* **403 on every admin call** — capability grant missing; check `/capabilities`.
* **Session expired immediately after sign-in** — cookie domain mismatch between sodmin and coauth; check `COAUTH_PUBLIC_URL` in `docker-entrypoint.sh`.
* **`/healthz/deep` returns 503** — soland is unreachable from sodmin; check `SOLAND_URL` and the network policy between the two containers.
* **`cx.error.capability_denied` on a button that's visible** — UI hiding is cosmetic only; the backend is the source of truth. Request the grant.

For deployment topology and the full port / path routing table see
[DEPLOYMENT.md](../DEPLOYMENT.md).
