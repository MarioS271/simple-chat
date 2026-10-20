# Data Flow

How data moves through a chat network, from starting a server to a message appearing in every client's TUI. For the
byte level format of what is sent, see [PACKETS.md](PACKETS.md). Every rule below is verified against the
implementation in [src/server](../src/server) and [src/client](../src/client).

## The model

A network consists of one server and any number of clients. The topology is a star: clients never talk to each other
directly, every packet goes through the server.

```
client A ---+
            |
client B ---+--- server (broadcast channel)
            |
client C ---+
```

The server does not store anything. There is no history, a client only sees messages that were broadcast while it was
connected. All packets in both directions are encrypted with the same pre-shared key of the server, see
[PACKETS.md](PACKETS.md).

## 1. Components

### 1.1. Server

| Part                | Role                                                                                |
|---------------------|-------------------------------------------------------------------------------------|
| Accept loop         | [start.rs](../src/server/cmd/start.rs), accepts TCP connections on `0.0.0.0:<port>` |
| Client task         | [handle_client.rs](../src/server/handle_client.rs), one tokio task per connection   |
| Broadcast channel   | Holds up to 256 packets, every client task subscribes to it                         |
| Names list          | Shared list of the names of all connected clients                                   |

### 1.2. Client

| Part             | Role                                                                                             |
|------------------|--------------------------------------------------------------------------------------------------|
| Main thread      | TUI, keyboard input, sends chat messages ([ui.rs](../src/client/ui.rs))                          |
| Heartbeat thread | Sends a heartbeat every 10 seconds                                                               |
| Receive thread   | Reads packets from the server into the shared state ([receive.rs](../src/client/net/receive.rs)) |
| Shared state     | The message list the TUI draws from ([state.rs](../src/client/state.rs))                         |

The clients use blocking std threads, the server uses async tokio tasks.

## 2. Connecting

```
client                                     server
  |  ----------- TCP connect ------------>   |
  |  <----------- Heartbeat --------------   |   server speaks first
  |  ----- Handshake(version, name) ----->   |   within 5 seconds
  |                                          |   checks, then registers the client
  |  <---- SystemMessage "x connected" ---   |   broadcast to everybody
```

1. The client opens the TCP connection and enables `TCP_NODELAY` and a 30 second read timeout.
2. The server immediately sends a heartbeat. The client requires this to be the first packet and exits if it is
   anything else. This also proves early that both sides use the same key.
3. The client sends a handshake with its protocol version and user name.
4. The server waits at most 5 seconds (`INITIAL_CONNECT_TIMEOUT`) for the handshake.
5. The server validates the handshake. On success the client is added to the names list and a "connected"
   system message is broadcast.
6. The client starts the TUI and its threads.

### 2.1. What the server checks

- Handshake arrives within 5 seconds    
- Decryption and deserialization succeed
- First packet is a handshake           
- Protocol version equals the server's  
- Name is 1 to 32 bytes after trimming  
- Name contains no control characters   
- Name is not already in use

> If any of the above checks fail, the server drops the connection.

The name comparison ignores ASCII case, so `Bob` and `bob` clash. The server sends no error packet back, a rejected
client only sees that the connection was closed ("Disconnected").

## 3. Sending a message

```
client A                       server                          client A, B, C
  |  ChatMessage(name, text)     |                                    |
  | ---------------------------> |  validate, overwrite name/time     |
  |                              |  broadcast ----------------------> |  every subscriber
  |                              |                                    |  shows it in the TUI
```

1. The user presses enter. The client builds a `ChatMessage` with its sender name and a timestamp of `0`, encrypts and
   frames it, and writes it to the socket. It does not show the message locally.
2. The server reads the frame, decrypts it and deserializes it.
3. The server applies the rules from section 3.1.
4. The server puts the packet into the broadcast channel.
5. Every client task, including the one of the sender, receives it from the channel, encrypts it again and writes it
   to its own socket.
6. Each client's receive thread decrypts it and pushes it into the shared state, and the TUI draws it.

The sender sees its own message only through step 5, so a message that appears in your TUI has made the full trip
through the server.

Each client task encrypts the packet on its own, so every recipient gets a different nonce and different ciphertext
for the same message.

### 3.1. What the server does to a chat message

| Step                      | Behavior                                                                          |
|---------------------------|-----------------------------------------------------------------------------------|
| Sender name               | If it differs from the handshake name it is replaced, the message is not rejected |
| Timestamp                 | Always overwritten with the server's clock, whatever the client sent              |
| Length                    | Over 1024 bytes of text disconnects the client                                    |
| Empty or whitespace only  | Disconnects the client                                                            |
| Control characters        | Disconnects the client                                                            |

The sender name and timestamp therefore cannot be forged. The server is the only source of time, so clock differences
between clients do not matter, but the server's time zone is not used: each client formats the unix timestamp in its
own local time zone.

### 3.2. Packet types a client may send

| Packet        | Result                              |
|---------------|-------------------------------------|
| Heartbeat     | accepted                            |
| ChatMessage   | validated and broadcast             |
| Handshake     | after the first one: client kicked  |
| SystemMessage | client kicked                       |

## 4. Heartbeats and timeouts

Both sides send a heartbeat every 10 seconds (`TCP_TIMEOUT` divided by three), and treat any received packet as proof
of life.

| Side   | Timeout                                         | Effect                                      |
|--------|-------------------------------------------------|---------------------------------------------|
| Server | no packet from a client for 30 seconds          | the client is disconnected                  |
| Server | a socket write takes longer than 30 seconds     | the client is disconnected                  |
| Client | no data from the server for 30 seconds          | prints "Receive Timed Out" and exits        |

Heartbeats are sent even while the chat is idle, so a quiet but healthy connection stays open. Heartbeats are never
broadcast to other clients, each connection has its own.

## 5. Disconnecting

A client leaves when it exits, loses the connection, times out or is kicked for breaking a rule.

1. The client task ends.
2. The server broadcasts a "disconnected" system message.
3. The name is removed from the names list, so it can be used again.

A client that was rejected during the handshake never made it into the names list and causes no system message.

On the client side, any receive error, deserialization error or decryption failure restores the terminal and exits
the process with a non-zero code. There is no automatic reconnect.

## 6. Backpressure

The broadcast channel holds 256 packets. Every client task reads from it as fast as its socket allows. If a client is
so slow that the channel overwrites packets it has not read yet, the server disconnects that client with a "Message
Overflow" error rather than skipping messages silently or slowing everybody else down.

## Gotchas

### The server never tells a client why it was rejected

Wrong protocol version, bad name, duplicate name, key mismatch and a handshake timeout all look the same from the
client: the connection closes. The reason is only printed on the server console.

### A wrong key looks like a dead server

The key is never transmitted, so a client with a wrong key cannot decrypt the first heartbeat. It prints "Failed to
decrypt message" and exits. The server in turn fails to decrypt anything that client sends and drops it.

### Messages sent while disconnected are lost

There is no queue or history. A client that reconnects sees an empty chat and only the new "connected" notice.
