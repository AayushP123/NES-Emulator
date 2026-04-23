// src/bin/nestest.rs
// Run with: cargo run --bin nestest -- nestest.nes nestest.log
//
// Boots nestest.nes at $C000 (automation mode, no PPU needed),
// logs every instruction in Nintendulator format, then diffs line by line
// against the official nestest.log and prints the first mismatch.

use nes_emulator::bus::Bus;
use nes_emulator::cart::Cart;
use nes_emulator::cpu::Cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).map(|s| s.as_str()).unwrap_or("nestest.nes");
    let log_path = args.get(2).map(|s| s.as_str()).unwrap_or("nestest.log");

    // Load ROM
    let rom  = std::fs::read(rom_path).expect("Could not read nestest.nes");
    let cart = Cart::from_bytes(&rom);
    let mut bus = Bus::new(cart);

    // Load official log lines for comparison
    let log_text  = std::fs::read_to_string(log_path).expect("Could not read nestest.log");
    let log_lines: Vec<&str> = log_text.lines().collect();

    // Automation mode: set PC to $C000, not the reset vector
    let mut cpu = Cpu { a: 0, x: 0, y: 0, pc: 0xC000, sp: 0xFD, p: 0x24 };

    let mut line_num  = 0usize;
    let mut mismatches = 0usize;

    loop {
        // Snapshot registers BEFORE executing
        let pc = cpu.pc;
        let a  = cpu.a;
        let x  = cpu.x;
        let y  = cpu.y;
        let p  = cpu.p;
        let sp = cpu.sp;

        // Build our register string
        let our_regs = format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            a, x, y, p, sp
        );

        // Compare against official log
        if let Some(expected) = log_lines.get(line_num) {
            // Extract just the register block from the official log line
            let expected_regs = extract_regs(expected);

            if our_regs != expected_regs {
                println!("MISMATCH at line {}:", line_num + 1);
                println!("  expected: {}", expected);
                println!("  got:      {:04X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                         pc, a, x, y, p, sp);
                mismatches += 1;
                // Stop after 5 so you're not spammed with cascading failures
                if mismatches >= 5 {
                    println!("\nStopped after 5 mismatches.");
                    break;
                }
            }
        } else {
            println!("Ran past end of nestest.log at line {}", line_num + 1);
            break;
        }

        line_num += 1;

        // Execute instruction and tick PPU 3x
        let alive = cpu.step(&mut bus);
        for _ in 0..3 { bus.ppu_tick(); }

        if !alive {
            println!("CPU hit BRK at line {}", line_num);
            break;
        }

        // nestest writes a non-zero result code to $02 when a test group fails
        let result = bus.read(0x0002);
        if line_num > 20 && result != 0x00 {
            println!("\nnestest FAILED:");
            println!("  $02 = {:02X}  (look this up in nestest.txt)", result);
            println!("  $03 = {:02X}", bus.read(0x0003));
            break;
        }

        // nestest.log is ~8991 lines
        if line_num >= 9000 { break; }
    }

    // Print final result
    println!("\n--- Result ---");
    let code02 = bus.read(0x0002);
    let code03 = bus.read(0x0003);
    println!("$02 = {:02X}  $03 = {:02X}  mismatches = {}", code02, code03, mismatches);
    if code02 == 0x00 && code03 == 0x00 && mismatches == 0 {
        println!("ALL TESTS PASSED");
    } else if mismatches == 0 {
        println!("Registers matched but nestest reported failures - look up codes in nestest.txt");
    } else {
        println!("Look up failure codes in nestest.txt");
    }
}

// Extract "A:xx X:xx Y:xx P:xx SP:xx" from a log line by finding each field individually
// This ignores the PPU/CYC columns that come after in the official log
fn extract_regs(line: &str) -> String {
    format!(
        "A:{} X:{} Y:{} P:{} SP:{}",
        field(line, "A:"),
        field(line, "X:"),
        field(line, "Y:"),
        field(line, "P:"),
        field(line, "SP:"),
    )
}

// Grab the 2 hex chars immediately after a label like "A:" or "SP:"
fn field<'a>(line: &'a str, label: &str) -> &'a str {
    if let Some(pos) = line.find(label) {
        let start = pos + label.len();
        let end   = (start + 2).min(line.len());
        &line[start..end]
    } else {
        "??"
    }
}