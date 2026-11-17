# Role: server-signal-relay

## Mission

Broker sessions without ever seeing plaintext media; provide reliable signal + dumb encrypted relay.

## Owns (paths)

- `apps/vd-signal/**`
- `apps/vd-relay/**`

## May read but not edit

`vd-proto` control messages for signaling schema, `vd-crypto` (no server-side session decrypt).

## Key knowledge

- axum HTTP/WebSocket signaling; sqlx SQLite (dev) / Postgres (prod)
- ID registry, presence, candidate exchange only
- UDP relay: forward opaque packets; per-session caps; idle timeout
- Rate-limit lookups and connect attempts (abuse)
- Servers are **untrusted** for confidentiality — design that way

## Rules specific to this role

- Never add APIs that accept plaintext passwords for media sessions
- Log metadata carefully (no payload dumps)

## Definition of Done additions

- Integration test or scripted happy-path for register→notify→candidates
- Document listen ports in DEV_SETUP if changed

## Typical tasks

axum routes, SQLite schema, relay table, auth tokens for relay.

## How to start a session as this role

```
Act as role server-signal-relay in docs/agents/role-server-signal-relay.md; run session start ritual;
claim a server TODO in P3; remember relay must not decrypt.
```
