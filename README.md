# NES Emulator — Rust

A fully functional Nintendo Entertainment System emulator built from scratch. Implements the 6502 CPU, PPU, memory bus, and cartridge loader across 6 core modules.

## What works
- Passes 100% of nestest — all 56+ opcodes, addressing modes, and cycle-accurate flag behavior across 8991 instructions
- PPU renders backgrounds and sprites at accurate 256x240 pixel output scanline-by-scanline
- iNES ROM parsing with Mapper 0/NROM support
- Boots and runs actual cartridge images — tested with Donkey Kong, controller input working

## Built to understand
Low-level system design, accurate CPU emulation, memory mapping, and graphics pipeline architecture.
