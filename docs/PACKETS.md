# Packet Structure

This doc describes how bytes look on the wire from a chat message typed into the TUI down to the TCP stream.

## The layers

A packet passes through four layers on its way out and the same four in reverse on its way in:

```
Packet (typed)
  -> serialize        packet type byte + payload
  -> pad              hide the real length
  -> encrypt          nonce + AES-256-GCM ciphertext + tag
  -> frame            4 byte length prefix
  -> TCP
```

| Layer      | Code                                                                                                                  |
|------------|-----------------------------------------------------------------------------------------------------------------------|
| Packet     | [packet.rs](../src/packets/packet.rs), [handshake.rs](../src/packets/handshake.rs), [message](../src/packets/message) |
| Padding    | `pad` / `unpad` in [encrypt.rs](../src/encrypt.rs)                                                                    |
| Encryption | `encrypt` / `decrypt` in [encrypt.rs](../src/encrypt.rs)                                                              |
| Framing    | [framing.rs](../src/packets/framing.rs)                                                                               |

All multi-byte integers are big endian.

## 1. Packet layer

A packet is one type byte followed by the payload of that type.

```
+-------------+------------------+
| type (1 B)  | payload (n B)    |
+-------------+------------------+
```

| Type | Name          | Payload                      | Direction          |
|------|---------------|------------------------------|--------------------|
| 0    | Heartbeat     | none                         | both               |
| 1    | Handshake     | see 2.1                      | client to server   |
| 2    | ChatMessage   | see 2.2                      | both               |
| 3    | SystemMessage | see 2.3                      | server to client   |

An unknown type byte or an empty buffer is rejected as invalid data.

### 1.1. Handshake

```
+--------------------+-----------------------+
| protocol_version   | client_name           |
| u16 (2 B)          | UTF-8, rest of packet |
+--------------------+-----------------------+
```

The name has no terminator or length field, it simply runs to the end of the packet. A packet with no name bytes is
rejected as a zero size name.

### 1.2. ChatMessage

```
+----------------+---------------------+------------------------+
| timestamp      | sender_name         | content                |
| u64 (8 B)      | 32 B, NUL padded    | UTF-8, rest of packet  |
+----------------+---------------------+------------------------+
```

The sender name is a fixed 32 byte field. Names shorter than 32 bytes are padded with `0x00`, names are cut at a
character boundary so a multi-byte character is never split. The timestamp is unix seconds. A client always sends
`0` here, the server overwrites it on arrival (see [DATA_FLOW.md](DATA_FLOW.md)).

### 1.3. SystemMessage

```
+----------------+------------------------+
| timestamp      | content                |
| u64 (8 B)      | UTF-8, rest of packet  |
+----------------+------------------------+
```

System messages carry join and leave notices. They have no sender field. A client that sends one is kicked.

### 1.4. Size of a serialized packet

| Packet        | Size in bytes                |
|---------------|------------------------------|
| Heartbeat     | 1                            |
| Handshake     | 1 + 2 + name length          |
| ChatMessage   | 1 + 8 + 32 + content length  |
| SystemMessage | 1 + 8 + content length       |

## 2. Padding

Before encryption the serialized packet is padded, so the ciphertext length does not reveal the exact message
length. Only the bucket is visible.

The padded length is the smallest of 100, 200, 300, ... that fits the data plus one byte:

```
data (n B) | zero bytes ... | padding_len (1 B)
```

The last byte stores how many bytes were added in total, including itself. The padding bytes in between are `0x00`.
On decryption the last byte is read and that many bytes are cut off. A padding length of `0` or one that is larger
than the data is rejected as invalid padding.

| Serialized packet size | Padded size |
|------------------------|-------------|
| 1 to 99                | 100         |
| 100 to 199             | 200         |
| 200 to 299             | 300         |
| 899                    | 900         |
| 900                    | 1000        |

A heartbeat (1 byte) and a handshake with a short name are therefore indistinguishable from a short chat message by
size alone.

## 3. Encryption

The padded data is encrypted with AES-256-GCM using the pre-shared key of the server (the `key` field in
`servers.toml` and `clients.toml`, base64 encoded, 32 bytes after decoding).

```
+------------------+-------------------------------+------------------+
| nonce (12 B)     | ciphertext (padded length)    | tag (16 B)       |
+------------------+-------------------------------+------------------+
```

- A fresh random nonce is drawn for every packet and prepended in the clear.
- The tag authenticates the ciphertext, so a packet that was modified or encrypted with a different key fails to
  decrypt.
- Data shorter than 28 bytes (nonce plus tag) is rejected before decryption is attempted.
- Both directions use the same key. There is no key exchange, the key is shared out of band (see `--show-key`).

The encrypted size is always the padded size plus 28 bytes, so a 100 byte bucket becomes 128 bytes.

## 4. Framing

TCP is a stream, so every encrypted packet is prefixed with its length:

```
+-------------------+----------------------------+
| length (u32, 4 B) | encrypted packet (length B)|
+-------------------+----------------------------+
```

The length counts only the encrypted packet, not the prefix itself.

| Side               | Implementation                                   | Frame limit                    |
|--------------------|--------------------------------------------------|--------------------------------|
| Client read/write  | [framing.rs](../src/packets/framing.rs)          | 2048 bytes                     |
| Server write       | [framing.rs](../src/packets/framing.rs)          | 2048 bytes                     |
| Server read        | tokio `LengthDelimitedCodec` (4 byte big endian) | 2048 bytes                     |

A frame over the limit is rejected before any body bytes are read or written.

Both implementations produce the same wire format and use the same limit. The largest legitimate frame, a 1024 byte
message, is 1128 bytes.

## 5. Complete example

A chat message with the text `hi` from a user called `bob`:

| Step      | Content                                              | Size |
|-----------|------------------------------------------------------|------|
| Packet    | `02` + timestamp (8) + `bob` + 29 NULs + `hi`        | 43   |
| Padded    | packet + 56 zero bytes + `39`                        | 100  |
| Encrypted | nonce (12) + ciphertext (100) + tag (16)             | 128  |
| Framed    | `00 00 00 80` + encrypted                            | 132  |

## Gotchas

### Length scrambling does not exist

The only length hiding is the padding in section 3. The 4 byte frame prefix is sent in plain big endian and shows the
ciphertext length, which reveals the padding bucket (but not the exact message length). There is no obfuscation of
the prefix itself.

### A single byte tamper kills the connection

Decryption failures and padding errors are not recoverable. The server returns an error for that client and closes
the connection, and the client exits.
