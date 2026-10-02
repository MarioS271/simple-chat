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

| Class          | Members                                                | On repeat                          | Stops the pass?          |
|----------------|--------------------------------------------------------|------------------------------------|--------------------------|
| Terminating    | `--help`, `--list`                                     | first wins                         | yes, returns immediately |
| Unknown option | any other `--xxx`                                      | n/a                                | yes, returns an error    |
| Mode           | `--add`, `--remove`, `--new`, `--delete`, `--show-key` | first wins, second is a hard error | no                       |
| Value          | `--port N` (server), `--address S` (client)            | last wins, silently                | no                       |
| Positional     | the server name                                        | last wins, silently                | no                       |

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

| Invocation                               | Result      |
|------------------------------------------|-------------|
| `server x --new --port 1 --port 2`       | port 2      |
| `client x --add --address a --address b` | address `b` |
| `server a b --show-key`                  | name `b`    |
| `server --show-key a b`                  | name `b`    |

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
    any other --xxx         -> RETURN error (unknown option)
    anything else           -> name = it (overwrites any previous name)

after the loop:
    1. no name AND no mode  -> help
    2. no name (mode set)   -> error "Missing 'name' argument"
    3. otherwise            -> dispatch on mode
                               (no mode -> Connect / Start)
```

Step 1 must precede step 2. It is what makes a bare `simple-chat server` print
help instead of failing.

Note that only the name is checked for emptiness. A value flag that was never
given stays at its empty default and is not reported.

## Gotchas

### Value flags swallow the next argument unconditionally

`--port` and `--address` take whatever follows them without checking whether it
looks like a flag. A flag placed directly after one of them is consumed as its
value and never reaches the match arm.

| Invocation                        | Result                           |
|-----------------------------------|----------------------------------|
| `server --port --help srv`        | Err "Invalid Port: --help"       |
| `server --port --new srv`         | Err "Invalid Port: --new"        |
| `server srv --port`               | Err "--port requires a value"    |
| `client --address --help x --add` | address becomes `--help:42003`   |
| `client x --add --address`        | Err "--address requires a value" |

This is the one case where `--help` does not win.

The two flags differ in how loudly they fail. `--port` validates its value as a
`u16`, so a swallowed flag produces an error. `--address` accepts any string, so
a swallowed flag is silently written into the config as a hostname.

### Value flags are silently ignored outside their own mode

Only `New` carries a port and only `Add` carries an address. The other modes do
not, so a value flag given alongside them is parsed, validated, and then
discarded without a word.

| Invocation                        | Result                       |
|-----------------------------------|------------------------------|
| `server x --delete --port 9999`   | deletes `x`, port ignored    |
| `client x --remove --address foo` | removes `x`, address ignored |

### A malformed port still errors even if the mode would ignore it

Validation happens during the pass, before the mode is known.

| Invocation                     | Result                  |
|--------------------------------|-------------------------|
| `server x --delete --port abc` | Err "Invalid Port: abc" |

### A missing `--address` is not an error

`--add` without `--address` leaves the address empty. The handler then appends
the default port to it, producing an address with no host.

| Invocation                           | Stored address  |
|--------------------------------------|-----------------|
| `client x --add --address 1.2.3.4:5` | `1.2.3.4:5`     |
| `client x --add --address 1.2.3.4`   | `1.2.3.4:42003` |
| `client x --add`                     | `:42003`        |

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

## Not supported

These are rejected as unknown options, so no priority applies:

| Form            | Example       |
|-----------------|---------------|
| short flags     | `-n`          |
| `--flag=value`  | `--port=1234` |
| clustered flags | `-abc`        |
