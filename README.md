# Gomoku AI (Rust)

A Gomoku (Five-in-a-Row) AI bot implemented in Rust.

## Building

```sh
make
```

This compiles the binary `pbrain-gomoku-ai` in the current directory.

## Running

The bot communicates via stdin/stdout using the Gomoku protocol. Run it with a game manager like `liskvork`:

```sh
./pbrain-gomoku-ai
```

Or manually for testing:
```sh
echo "START 20" | ./pbrain-gomoku-ai
echo "BEGIN" | ./pbrain-gomoku-ai
```

## Supported Commands

The bot implements the mandatory protocol commands:
- `START <size>` - Initialize game (only size 20 supported)
- `TURN <x>,<y>` - Opponent played at coordinates, bot responds with move
- `BEGIN` - Bot plays first move
- `BOARD` - Prefill board state (lines: `x,y,field` or `DONE`)
- `INFO <key> <value>` - Game information (ignored)
- `ABOUT` - Returns bot information
- `RESTART` - Reset game state
- `END` - Exit bot

## Debug Options

Enable debug logging to stderr with:
```sh
GOMOKU_DEBUG=1 ./pbrain-gomoku-ai
```

## Constraints

- Time limit: 5 seconds per move
- Memory limit: 70 MB max
- Board size: 20x20
- Standard library only

## Project Details

See [CONSTRAINTS.md](./CONSTRAINTS.md) for full requirements and acceptance criteria.
