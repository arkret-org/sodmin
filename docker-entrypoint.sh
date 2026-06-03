#!/bin/sh
set -eu

SOLAND_URL="${SOLAND_URL:-}"
COAUTH_URL="${COAUTH_URL:-}"
COAUTH_PUBLIC_URL="${COAUTH_PUBLIC_URL:-}"
SODMIN_PORT="${SODMIN_PORT:-80}"

RESOLVERS="$(awk '/^nameserver / { print $2 }' /etc/resolv.conf | paste -sd ' ' -)"
LOOKUP_UNAVAILABLE=0

if [ -z "$SOLAND_URL" ]; then
    echo "SOLAND_URL must be set" >&2
    exit 1
fi

extract_host() {
    url="$1"
    rest="${url#*://}"
    authority="${rest%%/*}"
    printf '%s\n' "${authority%%:*}"
}

can_resolve_url_host() {
    host="$(extract_host "$1")"
    [ -n "$host" ] || return 1
    [ "$LOOKUP_UNAVAILABLE" -eq 0 ] || return 1

    if timeout 0.5 getent hosts "$host" >/dev/null 2>&1; then
        return 0
    fi

    LOOKUP_UNAVAILABLE=1
    return 1
}

write_proxy_location() {
    location_path="$1"
    target_url="$2"
    auth_header="${3:-}"

    cat >> /etc/nginx/conf.d/default.conf <<EOF
    location ${location_path} {
        proxy_pass ${target_url};
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
EOF

    if [ -n "$auth_header" ]; then
cat >> /etc/nginx/conf.d/default.conf <<EOF
        proxy_set_header Authorization ${auth_header};
EOF
    fi

cat >> /etc/nginx/conf.d/default.conf <<EOF
    }
EOF
}

write_dynamic_proxy_location() {
    location_path="$1"
    variable_name="$2"
    auth_header="${3:-}"

    cat >> /etc/nginx/conf.d/default.conf <<EOF
    location ${location_path} {
        proxy_pass \$${variable_name};
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
EOF

    if [ -n "$auth_header" ]; then
cat >> /etc/nginx/conf.d/default.conf <<EOF
        proxy_set_header Authorization ${auth_header};
EOF
    fi

cat >> /etc/nginx/conf.d/default.conf <<EOF
    }
EOF
}

# Emit /config.json consumed by the Dioxus runtime.
printf '{"coauth_public_url":"%s"}' \
    "$COAUTH_PUBLIC_URL" \
    > /usr/share/nginx/html/config.json

cat > /etc/nginx/conf.d/default.conf <<EOF
server {
    listen ${SODMIN_PORT};
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    # Hardening headers. CSP intentionally omits 'unsafe-inline' on
    # script-src; 'wasm-unsafe-eval' is required for the Dioxus WASM
    # bundle. Adjust connect-src if the deployment fronts additional
    # services beyond the same-origin proxy paths below.
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "no-referrer" always;
    add_header Permissions-Policy "geolocation=(), microphone=(), camera=(), payment=()" always;

    # Healthz actually serves index.html so liveness fails (503) when the
    # wasm bundle disappears from disk, instead of nginx happily returning
    # a static 'ok' over an empty document root. wget --spider only checks
    # the status line so the response body is not transferred.
    location = /healthz {
        access_log off;
        try_files /index.html =503;
    }

    # P5 — deep healthcheck that also verifies upstream soland
    # connectivity. Container orchestrators (k8s, nomad) can use this
    # as a readiness probe so a sodmin pod is not advertised "ready"
    # while soland is unreachable. Errors (DNS failure, network down,
    # soland returning anything other than 2xx/3xx) return 503 via
    # nginx error_page so the probe fails as expected.
    location = /healthz/deep {
        access_log off;
        proxy_pass ${SOLAND_URL}/healthz;
        proxy_set_header Host \$host;
        proxy_connect_timeout 2s;
        proxy_read_timeout 2s;
        proxy_intercept_errors on;
        error_page 500 502 503 504 =503 /healthz-upstream-down;
    }
    location = /healthz-upstream-down {
        access_log off;
        internal;
        return 503 "upstream soland unreachable\n";
    }
EOF

if [ -n "$COAUTH_URL" ]; then
    if can_resolve_url_host "$COAUTH_URL"; then
        write_proxy_location "/auth/" "$COAUTH_URL"
        write_proxy_location "/api/v1/auth/" "$COAUTH_URL"
        write_proxy_location "/api/admin/" "$COAUTH_URL" "\$http_authorization"
        write_proxy_location "/authorize" "$COAUTH_URL"
        write_proxy_location "/oauth2/" "$COAUTH_URL"
        write_proxy_location "/.well-known/" "$COAUTH_URL"
    else
        [ -n "$RESOLVERS" ] || RESOLVERS="127.0.0.11"
        cat >> /etc/nginx/conf.d/default.conf <<EOF
    resolver ${RESOLVERS} valid=30s ipv6=off;
    set \$coauth_backend ${COAUTH_URL};
EOF
        write_dynamic_proxy_location "/auth/" "coauth_backend"
        write_dynamic_proxy_location "/api/v1/auth/" "coauth_backend"
        write_dynamic_proxy_location "/api/admin/" "coauth_backend" "\$http_authorization"
        write_dynamic_proxy_location "/authorize" "coauth_backend"
        write_dynamic_proxy_location "/oauth2/" "coauth_backend"
        write_dynamic_proxy_location "/.well-known/" "coauth_backend"
    fi
fi

if can_resolve_url_host "$SOLAND_URL"; then
    write_proxy_location "/api/v1/events/" "$SOLAND_URL"
    write_proxy_location "/api/v1/sync/" "$SOLAND_URL"
    write_proxy_location "/api/v1/directory/" "$SOLAND_URL"
    # soland admin surface lives at the bare deployment-local `/admin/*`
    # namespace (canonical + operator + collection), per contrix-spec
    # service-http-binding.md §2.1. Distinct from coauth's `/api/admin/*`
    # auth-admin surface above — the two prefixes do not overlap.
    write_proxy_location "/admin/" "$SOLAND_URL"
else
    if ! grep -q "resolver " /etc/nginx/conf.d/default.conf; then
        [ -n "$RESOLVERS" ] || RESOLVERS="127.0.0.11"
        cat >> /etc/nginx/conf.d/default.conf <<EOF
    resolver ${RESOLVERS} valid=30s ipv6=off;
EOF
    fi
    cat >> /etc/nginx/conf.d/default.conf <<EOF
    set \$soland_backend ${SOLAND_URL};
EOF
    write_dynamic_proxy_location "/api/v1/events/" "soland_backend"
    write_dynamic_proxy_location "/api/v1/sync/" "soland_backend"
    write_dynamic_proxy_location "/api/v1/directory/" "soland_backend"
    # soland admin surface at the bare `/admin/*` namespace (see the
    # resolvable branch above for rationale).
    write_dynamic_proxy_location "/admin/" "soland_backend"
fi

cat >> /etc/nginx/conf.d/default.conf <<EOF
    location ~* \.(wasm|js|css|png|jpg|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    location / {
        try_files \$uri \$uri/ /index.html;
    }
}
EOF

nginx -t
exec nginx -g "daemon off;"
