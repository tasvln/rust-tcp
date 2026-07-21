# Among Us TCP Server

A small authoritative multiplayer game server written in Rust, built to learn real-time networking fundamentals — the kind of client/server architecture used under the hood in social deduction games like Among Us.

Built incrementally from a raw TCP echo server up through a fully framed, authoritative, multi-client game server: connection handling, structured protocol design, authoritative state, state sync vs. event relay, and clean disconnect handling.

## Architecture

```
        ┌─────────────────────────┐
        │         SERVER          │  authoritative state: HashMap<Uuid, PlayerState>
        │                         │  ticks a full state broadcast every 500ms
        └────────────┬────────────┘
        ┌────────────┼────────────┐
        │            │            │
    ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
     Client A    Client B    Client C     send intents (Move, Chat, CompleteTask)
    └─────────┘ └─────────┘ └─────────┘   receive: Welcome, PlayerEvent, StateUpdate, PlayerLeft
```

Clients never declare what happened — they send *intent* (`ClientMessage`), and the server is the sole authority over the resulting state (`PlayerState`). This is the same authoritative-server model used by most real-time multiplayer games: clients can't be trusted to self-report position, task completion, etc., so the server always has the final say.

### Two sync strategies, used deliberately for different data

- **Event relay** (`tx` / `ClientMessage`, wrapped and rebroadcast as `ServerMessage::PlayerEvent`) — every individual message is relayed as it happens. Used for things where losing an individual message matters (e.g. chat).
- **State sync** (`state_tx` / `ServerMessage::StateUpdate`) — the server broadcasts a full snapshot of all player state on a fixed 500ms tick, independent of what triggered any particular change. Used for position/task data, where only the *current* value matters, not the history of how it got there. This is what allows a client that joins late to immediately see accurate state for every player, rather than only learning about moves that happen after they connect.

### Message framing

TCP is a byte stream with no built-in message boundaries. Early in development, sending two messages back-to-back could result in both arriving in a single `read()` call, with the second message's bytes silently discarded by the deserializer — a real bug hit and fixed during development (see `protocol::framing`). Every message, in both directions, is now sent with a 4-byte length prefix before the payload, and the receiver reads exactly that many bytes before attempting to deserialize.

### Concurrency model

- One `tokio::spawn` per connected client for reading their messages.
- Two additional spawned tasks per client: one forwarding the event-relay broadcast to that client, one forwarding the state-sync broadcast — each client's socket writer is wrapped in `Arc<Mutex<...>>` since multiple tasks write to it concurrently.
- Shared authoritative state (`HashMap<Uuid, PlayerState>`) is wrapped in `Arc<Mutex<...>>`, giving every connection task shared, safely-synchronized access.
- Failed writes (e.g. to a disconnected client) end that forwarding task quietly rather than panicking — a client disconnecting is an expected, routine event, not an error condition.

## Protocol

```rust
enum ClientMessage {
    Move { id: Uuid, dx: f32, dy: f32 },
    Chat { id: Uuid, text: String },
    CompleteTask { id: Uuid, task_id: u32 },
}

enum ServerMessage {
    Welcome { player_id: Uuid },              // sent once, on connect
    StateUpdate { players: HashMap<Uuid, PlayerState> }, // ticked every 500ms
    PlayerEvent(ClientMessage),                // relayed as it happens
    PlayerLeft { player_id: Uuid },            // sent on disconnect
}

struct PlayerState {
    id: Uuid,
    x: f32,
    y: f32,
    completed_tasks: HashSet<u32>,  // HashSet, not Vec — a task can't be "completed" twice
}
```

Messages are serialized with `bincode` and framed with a 4-byte big-endian length prefix (`protocol::framing`).

## Running it

```bash
cargo run --bin server
# in a separate terminal:
cargo run --bin client
```

Run multiple clients in separate terminals to see state sync and event relay across connections. Killing a client (Ctrl+C) triggers a clean disconnect, visible as a `PlayerLeft` broadcast on every other connected client.

## Outcome of this project

- Async networking in Rust with `tokio` (listeners, streams, spawned tasks)
- Designing a wire protocol from scratch with `serde`/`bincode`
- Authoritative server architecture (never trusting client-reported state)
- Correct handling of TCP's lack of message boundaries (length-prefixed framing)
- Safe concurrent shared state (`Arc<Mutex<...>>`) across many connection tasks
- Broadcast channels (`tokio::sync::broadcast`) for one-to-many message distribution
- Graceful handling of disconnects/write failures instead of panicking

## Possible extensions (not implemented — kept out of scope deliberately)

- UDP version — would require building a custom reliability layer for messages that need guaranteed delivery, since UDP provides no ordering or delivery guarantees
- Kill/sabotage/meeting mechanics — more `ClientMessage` variants and state logic, no new networking concepts
- Actual rendering instead of console output
