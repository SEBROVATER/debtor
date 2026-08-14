# Caddy Edge Verification Runbook

This runbook captures the reproducible evidence required by ADR 0003 before Debtor production rollout. It uses Caddy `2.11.2` and the committed `deploy/Caddyfile.example` template.

## Inputs To Record

- Caddy version or container image digest: `caddy:2.11.2`, plus the digest used in the environment.
- Caddyfile path and commit SHA.
- Test origin backed by DNS or an `/etc/hosts` entry, for example `https://debtor.example.test`.
- A test certificate trusted by the client, or its CA path for `curl --cacert`.
- Private Debtor backend host/port, for example `127.0.0.1:3000`; replace the literal `reverse_proxy` host:port in the template and do not use an `http://` or `https://` URL.
- Caddy backend source CIDR configured in `APP_TRUSTED_PROXY_CIDRS`.
- Debtor forwarding mode: `APP_TRUSTED_PROXY_HEADER=x-forwarded-for`.
- Test client IP range, with real client addresses redacted from committed evidence.
- Linux client firewall access for the controlled UDP/443 block test, with cleanup privileges.

Use sentinel values only. Do not use real credentials, cookies, session IDs, CSRF tokens, submission tokens, provider URLs, query strings, or client identities in captured evidence.

## 1. Validate Caddy Version And Config

```bash
docker run --rm caddy:2.11.2 caddy version
docker run --rm \
  -v "$PWD/deploy/Caddyfile.example:/etc/caddy/Caddyfile:ro" \
  caddy:2.11.2 \
  caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
```

Expected evidence:

- Version output identifies Caddy `2.11.2`.
- Validation exits successfully.
- The adapted config keeps `servers { 0rtt off }`, strips untrusted forwarding headers, and uses HTTP/1.1 upstream transport.

## 2. Verify Private HTTP/1.1 Backend And Reuse

Start Debtor privately and Caddy with the selected environment values. The Caddy container uses host networking here so `127.0.0.1:3000` remains the private backend:

```bash
APP_BIND=127.0.0.1:3000 \
APP_TRUSTED_PROXY_CIDRS=127.0.0.1/32 \
APP_TRUSTED_PROXY_HEADER=x-forwarded-for \
cargo run
```

```bash
DEBTOR_SITE=debtor.example.test \
docker run --rm --network host \
  --env DEBTOR_SITE \
  -v "$PWD/deploy/Caddyfile.example:/etc/caddy/Caddyfile:ro" \
  caddy:2.11.2 \
  caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
```

Then issue repeated probe traffic through the edge:

```bash
curl --http2 -fsS https://debtor.example.test/healthz
curl --http3 -fsS https://debtor.example.test/healthz
```

Expected evidence:

- Debtor is reachable only through the private backend bind or private container network.
- Caddy upstream telemetry or adapted config shows `versions 1.1` and keepalive enabled.
- No backend TLS, QUIC, UDP, or HTTP/3 listener is required from Debtor.

## 3. Verify Forwarding Sanitation And Cross-Protocol Parity

Send spoofed forwarding input over HTTP/3 and fallback. The following flow obtains a fresh session-backed CSRF and submission token before each wrong-password attempt, then consumes the same limiter budget from each transport. Use a disposable test administrator configuration and stop before affecting any shared environment.

```bash
set -eu
origin='https://debtor.example.test'
spoof_headers=(
  -H 'X-Forwarded-For: 203.0.113.250, 198.51.100.250'
  -H 'Forwarded: for=203.0.113.251;proto=http'
  -H 'X-Forwarded-Host: attacker.invalid'
  -H 'X-Forwarded-Port: 8443'
  -H 'X-Forwarded-Server: attacker.invalid'
  -H 'X-Real-IP: 203.0.113.252'
  -H 'Client-IP: 203.0.113.253'
  -H 'True-Client-IP: 203.0.113.254'
  -H 'CF-Connecting-IP: 203.0.113.255'
  -H 'X-Cluster-Client-IP: 198.51.100.254'
)

failed_login() {
  label="$1"
  shift
  cookie="/tmp/debtor-${label}.cookies"
  form="/tmp/debtor-${label}.login.html"
  curl "$@" -fsS -c "$cookie" -b "$cookie" "${spoof_headers[@]}" "$origin/login" >"$form"
  csrf="$(python3 - "$form" <<'PY'
import re
import sys

html = open(sys.argv[1], encoding="utf-8").read()
print(re.search(r'name="csrf" value="([^"]+)"', html).group(1))
PY
)"
  submission_token="$(python3 - "$form" <<'PY'
import re
import sys

html = open(sys.argv[1], encoding="utf-8").read()
print(re.search(r'name="submission_token" value="([^"]+)"', html).group(1))
PY
)"
  curl "$@" -sS -o "/tmp/debtor-${label}.response" -w '%{http_code}\n' \
    -c "$cookie" -b "$cookie" "${spoof_headers[@]}" \
    --data-urlencode "csrf=${csrf}" \
    --data-urlencode "submission_token=${submission_token}" \
    --data-urlencode 'password=wrong-sentinel' \
    "$origin/login"
}

failed_login h3 --http3
failed_login fallback --http2
```

Repeat `failed_login h3 --http3` and `failed_login fallback --http2` until the bounded limiter response is observed. Compare only status/retry categories; do not record resolved client identities or limiter keys.

Expected evidence:

- Caddy strips attacker-supplied forwarding headers before the backend request.
- Debtor sees the same trusted client identity and login-limiter budget over HTTP/3 and HTTP/2 or HTTP/1.1 fallback.
- Captured application logs do not include client IPs, forwarding chains, cookies, session IDs, CSRF tokens, submission tokens, or query strings. The spoof set above covers every forwarding/proxy-identity header deleted by the template.

## 4. Verify Early-Data Replay Safety

ADR 0003 requires Caddy `0rtt off`, so QUIC 0-RTT must be disabled at the selected edge. Also prove the defense-in-depth matcher rejects marked early-data requests before Debtor dispatch if an upstream edge reintroduces `Early-Data: 1`.

```bash
curl --http3 -i \
  -H 'Early-Data: 1' \
  -X POST \
  --data 'csrf=sentinel&password=sentinel' \
  https://debtor.example.test/login
```

Expected evidence:

- Caddy config validation or adaptation shows `0rtt off`.
- The marked unsafe request receives `425 Too Early` from Caddy.
- Debtor access/application logs show no backend dispatch for the marked unsafe request.
- `GET /login` and authenticated HTML are not allow-listed for early data merely because they use a safe HTTP method.

Allowed marked early-data paths are limited to `GET` or `HEAD` requests for session-free replay-safe probes and immutable static assets:

```bash
curl --http3 -i -H 'Early-Data: 1' https://debtor.example.test/healthz
curl --http3 -i -H 'Early-Data: 1' https://debtor.example.test/readyz
curl --http3 -i -H 'Early-Data: 1' https://debtor.example.test/static/css/app.css
```

## 5. Verify Body Limits Before Backend Mutation Dispatch

Generate oversized sentinel payloads locally:

```bash
python3 - <<'PY' >/tmp/debtor-login-oversize.form
print('password=' + 'x' * 9000)
PY

python3 - <<'PY' >/tmp/debtor-form-oversize.form
print('field=' + 'x' * 270000)
PY
```

Send them through Caddy:

```bash
curl --http3 -i \
  -X POST \
  --data-binary @/tmp/debtor-login-oversize.form \
  https://debtor.example.test/login

curl --http3 -i \
  -X POST \
  --data-binary @/tmp/debtor-form-oversize.form \
  https://debtor.example.test/groups
```

Expected evidence:

- `/login` bodies over 8 KiB are rejected at or before Caddy with no password verification.
- Other unsafe form bodies over 256 KiB are rejected at or before Caddy with no mutation dispatch.
- Application logs contain only safe route/status categories, not body contents or submitted values.

## 6. Verify HTTP/3 Rollout And Fallback

Check initial `Alt-Svc` behavior and UDP reachability:

```bash
curl --http2 -I https://debtor.example.test/healthz
curl --http3 -I https://debtor.example.test/healthz
```

From a controlled Linux test client, block outbound UDP/443 to the resolved edge address, repeat the request with verbose protocol output, and always restore the rule:

```bash
edge_ip=203.0.113.10
sudo iptables -I OUTPUT -p udp -d "$edge_ip" --dport 443 -j REJECT
trap 'sudo iptables -D OUTPUT -p udp -d "$edge_ip" --dport 443 -j REJECT' EXIT
curl --http2 -v -fsS https://debtor.example.test/healthz 2>&1 | tee /tmp/debtor-fallback.txt
rg -F 'using HTTP/2' /tmp/debtor-fallback.txt
```

Expected evidence:

- Initial responses advertise the short `Alt-Svc: h3=":443"; ma=300` lifetime; no longer lifetime is configured in the repository template.
- UDP/443 reaches the edge before HTTP/3 is promoted.
- When UDP/443 is blocked, the same route succeeds over HTTP/2 or HTTP/1.1 fallback.
- Forwarding identity and limiter behavior match between HTTP/3 and fallback.

## 7. Verify Timeout Safety

Inspect the pinned active Caddy configuration and observe the established private upstream connection:

```bash
docker run --rm \
  -v "$PWD/deploy/Caddyfile.example:/etc/caddy/Caddyfile:ro" \
  caddy:2.11.2 \
  caddy adapt --config /etc/caddy/Caddyfile --adapter caddyfile

sudo ss -tnp state established '( dport = :3000 )' > /tmp/debtor-upstream-before.txt
for request in $(seq 1 10); do
  curl --http2 -fsS https://debtor.example.test/healthz >/dev/null
done
sudo ss -tnp state established '( dport = :3000 )' > /tmp/debtor-upstream-after.txt
diff -u /tmp/debtor-upstream-before.txt /tmp/debtor-upstream-after.txt
```

Expected evidence:

- The adapted config contains `dial_timeout 5s`, no response-header/read/write/stream/request timeout, and HTTP/1.1 keepalive.
- Repeated requests use an established private backend connection; capture the `ss` output with the test evidence.
- No Caddy timeout can terminate an admitted post-dispatch mutation before Debtor returns a definitive result.
- Real slow-mutation evidence is deferred to Story 2.1, because Story 1.10 has no real ledger mutation route to exercise.

## 8. Verify Safe Diagnostics

Run the preceding checks with sentinel header, body, password, and query values while capturing Debtor stderr. Assert that the sentinels do not appear:

```bash
RUST_LOG=debtor=debug,tower_http=debug cargo run 2> /tmp/debtor-edge.log

forbidden='attacker.invalid|203\.0\.113\.|wrong-sentinel|query-sentinel|provider\.invalid|session-sentinel|csrf-sentinel|submission-sentinel'
if rg -n "$forbidden" /tmp/debtor-edge.log; then
  printf '%s\n' 'forbidden sentinel found in application logs' >&2
  exit 1
fi
```

Expected evidence:

- Logs contain safe operation names, route patterns, status codes, bounded categories, startup/shutdown stages, and readiness categories only.
- Logs do not contain credentials, password hashes, cookies, session IDs, CSRF tokens, submission tokens, login limiter keys, client IPs, forwarding chains, query strings, provider URLs, SQL/database messages, monetary values, entity identifiers, raw adapter diagnostics, or request-derived values.

## Go/No-Go Rule

Production rollout is blocked until every required evidence item above is captured for the selected environment. If any check fails, keep Debtor private, fix the edge configuration or selected product/version, and rerun the full matrix before increasing HTTP/3 advertisement or routing real administrator traffic through the edge.
