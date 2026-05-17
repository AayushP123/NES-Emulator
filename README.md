# NES Emulator

Nintendo Entertainment System emulator written in Rust. Implements the 6502 CPU, PPU rendering path, memory bus, and iNES cartridge loading with a focus on correctness, debuggability, and low-level systems learning.

Built to understand how an 8-bit console actually moves from opcodes and memory maps to pixels on screen.

## Quick start

Requires Rust 2021 and a local `.nes` ROM image.

```bash
cargo run --release -- <path-to-rom.nes>
```

The emulator opens a fixed-size window at 3x NES resolution using `pixels` and `winit`.

## Validation

The repo includes an automated `nestest` runner for comparing CPU execution against the official Nintendulator log.

```bash
cargo run --bin nestest -- nestest.nes nestest.log
```

The runner boots `nestest.nes` at `$C000`, snapshots registers before each instruction, compares against `nestest.log`, and reports the first mismatches.

## What works

- 6502 CPU state, instruction fetch/decode/execute loop, stack behavior, status flags, and interrupts
- Addressing modes across zero-page, absolute, indexed, indirect, and branch paths
- PPU scanline/dot ticking with a 256x240 framebuffer
- Background and sprite rendering into RGBA output
- CPU/PPU bus wiring, DMA stalls, NMI polling, and memory-mapped PPU registers
- iNES cartridge parsing with Mapper 0 / NROM support
- `nestest` regression path for CPU correctness checks

## Structure

```text
src/
  main.rs              - window setup, ROM loading, CPU/PPU frame loop
  lib.rs               - library module exports
  cpu.rs               - 6502 register state, addressing modes, opcode execution
  bus.rs               - CPU memory map, cartridge/PPU wiring, DMA and NMI handling
  ppu.rs               - NES PPU state, scanline timing, framebuffer rendering
  cart.rs              - iNES parsing and cartridge PRG/CHR access
  bin/
    nestest.rs         - CPU regression runner against nestest.log
nestest.nes            - CPU validation ROM
nestest.log            - reference execution log
```

## Notes

This is intentionally a from-scratch emulator rather than a wrapper around an existing core. The code favors explicit CPU/PPU state and readable hardware boundaries so individual subsystems can be tested, fixed, and extended independently.
