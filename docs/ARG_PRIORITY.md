# Argument Priority

What wins when arguments conflict, repeat, or are combined in ways that make no
sense. Every rule below is verified against the implementation.

## The model

Arguments are processed in a single left-to-right pass. Each argument is
classified by its own text, never by its position, and each class has its own
conflict behaviour. Nothing is re-examined afterwards.

This means two different kinds of priority exist:

1. **Positional priority:** which *occurrence* of a repeated argument wins.
2. **Class priority:** which *kind* of argument wins over another kind.

Class priority is decided by where the parser stops, so it is only
order-independent for classes that do not stop the pass.

## Rules by argument class

| Class          | Members                                                 | On repeat                          | Stops the pass?          |
|----------------|---------------------------------------------------------|------------------------------------|--------------------------|
| Terminating    | `--help`, `--list`                                      | first wins                         | yes, returns immediately |
| Unknown option | any other `--xxx`                                       | n/a                                | yes, returns an error    |
| Mode           | `--add`, `--remove`, `--new`, `--delete`, `--show-key`  | first wins, second is a hard error | no                       |
| Value          | `--port N` (server), `--address S`, `--name S` (client) | last wins, silently                | no                       |
| Positional     | the server name                                         | last wins, silently                | no                       |

### Terminating flags: first wins, and they end the pass

`--help` and `--list` return from inside the loop. Nothing after them is ever
read, so they beat everything to their right, including errors.

| Invocation              | Result             | Why                      |
|-------------------------|--------------------|--------------------------|
| `server --list --bogus` | lists              | `--bogus` never examined |
| `server --bogus --list` | Err unknown option | `--bogus` hit first      |
| `server --list --help`  | lists              | `--list` came first      |
| `server --help --list`  | help               | `--help` came first      |
| `client x --add --list` | lists              | mode discarded           |

A terminating flag therefore has priority over everything to its right, and
over nothing to its left. It is the only class whose outcome depends on
position.

### Mode flags: first wins, second is fatal

Modes are mutually exclusive, so the second one is not overridden, it is
rejected. The error names the mode that was already set.

| Invocation                | Result                               |
|---------------------------|--------------------------------------|
| `server x --new --delete` | Err "Cannot use --delete with --new" |
| `server x --delete --new` | Err "Cannot use --new with --delete" |
| `client x --add --remove` | Err "Cannot use --remove with --add" |

### Value and positional: last wins, silently

Both are unconditional assignments, so a later occurrence overwrites an earlier
one. No warning is emitted.

| Invocation                               | Result        |
|------------------------------------------|---------------|
| `server x --new --port 1 --port 2`       | port 2        |
| `client x --add --address a --address b` | address `b`   |
| `client x --name a --name b`             | user name `b` |
| `server a b --show-key`                  | name `b`      |
| `server --show-key a b`                  | name `b`      |

### Mode and positional are order-independent

Neither stops the pass, so a mode flag and the name can appear in any order
relative to each other:

| Form A             | Form B             |
|--------------------|--------------------|
| `server srv --new` | `server --new srv` |
| `client srv --add` | `client --add srv` |

## Resolution algorithm

```
for each argument, left to right:
    --help / --list         -> RETURN that command immediately
    a mode flag             -> if no mode set: set it
                               else:           RETURN error (conflict)
    --port / --address      -> take the NEXT argument as the value
                               (overwrites any previous one)
    --name                  -> take the NEXT argument as the user name
                               (overwrites any previous one)
    any other --xxx         -> RETURN error (unknown option)
    anything else           -> name = it (overwrites any previous name)

after the loop:
    1. no name AND no mode  -> help
    2. no name (mode set)   -> error "Missing 'name' argument"
    3. Add mode, no address -> error "No server address given"
    4. otherwise            -> dispatch on mode
                               (no mode -> Connect / Start)
       Connect with a --name over the length limit
                            -> error "Username is too long"
```

Step 1 must precede step 2. It is what makes a bare `simple-chat server` print
help instead of failing.

The name is always checked for emptiness. The address is checked only in Add
mode, and the user name is only checked (for length) when connecting. A value
flag that was never given stays at its empty default.

## Gotchas

### Value flags swallow the next argument unconditionally

`--port`, `--address` and `--name` take whatever follows them without checking whether it
looks like a flag. A flag placed directly after one of them is consumed as its
value and never reaches the match arm.

| Invocation                        | Result                           |
|-----------------------------------|----------------------------------|
| `server --port --help srv`        | Err "Invalid Port: --help"       |
| `server --port --new srv`         | Err "Invalid Port: --new"        |
| `server srv --port`               | Err "--port requires a value"    |
| `client --address --help x --add` | address becomes `--help:42003`   |
| `client x --add --address`        | Err "--address requires a value" |
| `client x --name`                 | Err "--name requires a value"    |
| `client --name --help x`          | user name becomes `--help`       |

This is the one case where `--help` does not win.

The flags differ in how loudly they fail. `--port` validates its value as a
`u16`, so a swallowed flag produces an error. `--address` and `--name` accept
any string, so a swallowed flag is silently used as a hostname or user name
(`--address` writes it into the config).

### Value flags are silently ignored outside their own mode

Only `New` carries a port, only `Add` carries an address, and only `Connect`
carries a user name. The other modes do not, so a value flag given alongside
them is parsed and then discarded without a word. `--port` is still validated
as a number first; `--name` is not length-checked unless connecting.

| Invocation                        | Result                           |
|-----------------------------------|----------------------------------|
| `server x --delete --port 9999`   | deletes `x`, port ignored        |
| `client x --remove --address foo` | removes `x`, address ignored     |
| `client x --add --address a --name b` | adds `x`, user name ignored  |
| `client x --address a`            | connects to `x`, address ignored |

### A malformed port still errors even if the mode would ignore it

Validation happens during the pass, before the mode is known.

| Invocation                     | Result                  |
|--------------------------------|-------------------------|
| `server x --delete --port abc` | Err "Invalid Port: abc" |

### `--add` requires a non-empty address

`--add` without `--address`, or with an empty one, is an error. The default
port is appended by the handler only when the address has no port.

| Invocation                           | Result                         |
|--------------------------------------|--------------------------------|
| `client x --add --address 1.2.3.4:5` | stores `1.2.3.4:5`             |
| `client x --add --address 1.2.3.4`   | stores `1.2.3.4:42003`         |
| `client x --add`                     | Err "No server address given"  |

### An empty `--name` means "not given"

`--name ""` is treated like an omitted `--name`, and the saved user name is
used. A name longer than the limit is an error, but only when connecting.

## Full conflict table

| Invocation                               | Wins        | Result                                |
|------------------------------------------|-------------|---------------------------------------|
| `server --list --bogus`                  | `--list`    | list                                  |
| `server --bogus --list`                  | `--bogus`   | Err unknown option                    |
| `server --help --list`                   | `--help`    | help                                  |
| `server --list --help`                   | `--list`    | list                                  |
| `client x --add --list`                  | `--list`    | list, mode dropped                    |
| `server x --new --delete`                | n/a         | Err conflict                          |
| `server x --delete --new`                | n/a         | Err conflict                          |
| `client x --add --remove`                | n/a         | Err conflict                          |
| `server a b`                             | `b`         | name `b`                              |
| `server a b --show-key`                  | `b`         | show key of `b`                       |
| `server x --port 1 --port 2 --new`       | `2`         | new `x` on port 2                     |
| `server --new --port 1 x --port 2`       | `2`, `x`    | new `x` on port 2                     |
| `client x --add --address a --address b` | `b`         | add `x` at `b:42003`                  |
| `client x --name a --name b`             | `b`         | connect to `x` as `b`                 |
| `client x --add --address a --name b`    | `--add`     | add `x`, user name ignored            |
| `client x --add`                         | n/a         | Err no server address                 |
| `client x --name`                        | n/a         | Err `--name` requires a value         |
| `server x --name a`                      | n/a         | Err unknown option (client-only flag) |
| `server x --delete --port 9999`          | `--delete`  | delete `x`, port ignored              |
| `client x --remove --address foo`        | `--remove`  | remove `x`, address ignored           |
| `server x --delete --port abc`           | n/a         | Err invalid port                      |
| `server --port --help x`                 | `--port`    | Err invalid port `--help`             |
| `client --address --help x --add`        | `--address` | add `x` at `--help:42003`             |
| `server srv --port`                      | n/a         | Err `--port` requires a value         |
| `client x --add --address`               | n/a         | Err `--address` requires a value      |
| `server --add`                           | n/a         | Err unknown option (client-only flag) |
| `client --port 1 x --add`                | n/a         | Err unknown option (server-only flag) |
| `server`                                 | n/a         | help (no name, no mode)               |
| `server --new`                           | n/a         | Err missing name                      |

## Top level

The first argument alone selects the subcommand, before any of the rules above
apply.

| Invocation                     | Result                           |
|--------------------------------|----------------------------------|
| `simple-chat`                  | help                             |
| `simple-chat --help`           | help                             |
| `simple-chat --version`        | protocol version                 |
| `simple-chat client ...`       | client parsing                   |
| `simple-chat server ...`       | server parsing                   |
| `simple-chat x`                | Err unknown option or subcommand |
| `simple-chat --version client` | version, rest never read         |

## Not supported

These are rejected as unknown options, so no priority applies:

| Form            | Example       |
|-----------------|---------------|
| short flags     | `-n`          |
| `--flag=value`  | `--port=1234` |
| clustered flags | `-abc`        |
