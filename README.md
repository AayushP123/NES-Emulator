NES Emulator in Rust. Work in Progress

This project is a Nintendo Entertainment System emulator written in Rust. The goal is to understand low level system design through accurate CPU execution, memory mapping, and instruction decoding. The emulator focuses on correctness, clarity, and step by step validation against known 6502 behavior.

Current scope
• 6502 CPU core with full register state
• 64 KB memory model
• Instruction fetch and decode loop
• Little endian word handling
• Stack pointer and status register management
• Deterministic step based execution
