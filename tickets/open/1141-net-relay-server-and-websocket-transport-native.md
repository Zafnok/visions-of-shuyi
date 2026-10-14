---
id: "1141"
title: "Networking, step 2: a small relay server with room codes, and a native WebSocket transport"
type: feature
milestone: "Engine: networking"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1140"]
nick_input: none
completed:
---

# 1141 — Relay server and native WebSocket transport

## Context

ADR-0067 §1, §3 and §5. A relay forwards messages between the players in
a room. Both players connect outward, so home routers are no problem, no
NAT traversal is needed, and the browser can join (1142).

It is the simplest internet play there is, and it is ours to host. A
$4–6 a month VPS runs it. Vendors have closed or shrunk in 2026, so we
don't depend on one.

## Nick input

None to build it. Running it publicly is a decision for when a game ships
online (0053). This ticket only gives a Docker image and instructions.

## Scope

**In:**
- `engine/crates/relay` (binary `engine-relay`, tokio plus
  `tokio-tungstenite`, MIT). Its features:
  - rooms by a short invite code;
  - a cap on players per room;
  - messages forwarded with the sender's peer id;
  - rate and size limits;
  - rooms close when empty;
  - no game logic and no message storage;
  - a health endpoint.
- `WsTransport` (native) implements `Transport` over a WebSocket to the
  relay. The reliable channel is the socket. The unreliable channel is
  also the socket, documented as "reliable in practice on this backend".
- `RelaySession` creates a room, joins by code and lists peers. It is the
  first session backend (1145 generalizes it).
- A `Dockerfile`, and `engine/docs/net-relay.md`: deploying to a small
  VPS with TLS (Caddy, Apache-2.0), and what it costs.
- Tests: two native clients and a relay in one test process play the
  toy lockstep game. Abuse tests cover oversize messages, flooding and a
  bad room code.

**Out (do not do):**
- the web client (1142);
- accounts (1148);
- running a public instance.

## Implementation steps

1. Write the relay server with unit tests of its room table.
2. Write the WebSocket transport.
3. Add the integration and abuse tests.
4. Add the Docker image and the docs.

## Acceptance criteria

- [ ] The integration and abuse tests pass in CI.
- [ ] The Docker image builds in CI.
- [ ] `cargo deny check` passes. TLS uses `rustls` with `ring`, never
      `aws-lc-rs`, because aws-lc-rs needs NASM on the GNU toolchain.
- [ ] All gates pass.

## Tests required

- Unit: rooms and limits.
- Integration: relay plus two clients.

## Completion notes
