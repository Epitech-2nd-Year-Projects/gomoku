# Gomoku project constraints & acceptance criteria

This document outlines the hard constraints, rules, and acceptance criteria for the Gomoku AI project.

## Hard constraints (pass/fail)

### Delivery
- [ ] **Binary name**: `pbrain-gomoku-ai`
- [ ] **Makefile**: Must verify compilation (if needed).
  - [ ] `re` rule
  - [ ] `clean` rule (Must NOT delete binary)
  - [ ] `fclean` rule
- [ ] **Compilation**: Must compile on Linux via Makefile.
- [ ] **Source files**: Include all source. Exclude binaries, temp files, obj files (use `.gitignore`).
- [ ] **Bonus**: Bonus files in `bonus/` directory.

### Technical limits
- [ ] **Memory limit**: 70 MB max per bot.
- [ ] **Time limit**: 5 seconds max per move.
- [ ] **Libraries**: Standard library ONLY. (No tensorflow, scikit-learn, etc.)
- [ ] **Forbidden moves**: Automatically leads to defeat.
- [ ] **Forbidden libraries**: Leads to elimination.

### Application
- [ ] **Language**: Any (Rust chosen), as long as it works on the target environment ("the dump").
- [ ] **Protocol**: Must comply with the official Epitech Gomoku AI protocol (mandatory commands).

## Game rules

- [ ] **Board**: 20x20 intersection points (Goban).
- [ ] **Ruleset**: Freestyle.
  - [ ] Win condition: 5 stones in a row (vertical, horizontal, diagonal).
- [ ] **Players**: 2 players (Black and White).

## Acceptance criteria (definition of done)

### 1. Protocol compliance
- [ ] Responds to mandatory commands correctly.
- [ ] Communicates via standard input/output.

### 2. Gameplay (play to win)
- [ ] Detects winning situations (4-in-a-row to make 5).
- [ ] Block immediate threats (opponent has 4-in-a-row).
- [ ] Does not crash on valid input.
- [ ] Handles invalid moves gracefully (or loses if self-inflicted).

### 3. Performance (outsmart local bots)
- [ ] Beat low-level bots.
- [ ] Beat medium-level bots.
- [ ] Maximize win rate.

## Brawl (optional - opt-in)
- [ ] Create `.brawl` file at root.
- [ ] Name format: `^[a-zA-Z0-9 _-]{5,16}$`
