use nes_emulator::bus::Bus;
use nes_emulator::cart::Cart;
use nes_emulator::cpu::Cpu;
use nes_emulator::ppu;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::LogicalSize;
use winit::window::WindowBuilder;
use pixels::{Pixels, SurfaceTexture};

fn main() {
    // Load ROM path from first command line arg: cargo run -- roms/game.nes
    let path = std::env::args().nth(1).expect("Usage: nes-emulator <rom.nes>");
    let rom  = std::fs::read(&path).expect("Could not read ROM file");
    let cart = Cart::from_bytes(&rom);

    let mut bus = Bus::new(cart);
    let mut cpu = Cpu { a: 0, x: 0, y: 0, pc: 0, sp: 0xFD, p: 0x24 };

    cpu.reset(&mut bus);

    // Window: NES native resolution scaled 3x
    let scale      = 3u32;
    let event_loop = EventLoop::new();
    let window     = WindowBuilder::new()
        .with_title("NES Emulator")
        .with_inner_size(LogicalSize::new(
            ppu::SCREEN_WIDTH  as u32 * scale,
            ppu::SCREEN_HEIGHT as u32 * scale,
        ))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let size    = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32, surface).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::MainEventsCleared => {
                // Run CPU + PPU until the PPU signals a completed frame
                let mut frame_ready = false;
                while !frame_ready {
                    if bus.dma_active() {
                        // DMA stalls the CPU; PPU keeps ticking independently
                        bus.dma_tick();
                    } else {
                        // Check for NMI from PPU before executing next instruction
                        if bus.poll_nmi() {
                            cpu.trigger_nmi(&mut bus);
                        }
                        cpu.step(&mut bus);
                    }

                    // PPU runs 3 clocks for every 1 CPU clock
                    for _ in 0..3 {
                        if bus.ppu_tick() {
                            frame_ready = true;
                        }
                    }
                }

                // Copy PPU framebuffer (ARGB u32) into pixels crate buffer (RGBA u8)
                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let argb = bus.ppu.framebuffer[i];
                    pixel[0] = ((argb >> 16) & 0xFF) as u8; // R
                    pixel[1] = ((argb >>  8) & 0xFF) as u8; // G
                    pixel[2] = ( argb        & 0xFF) as u8; // B
                    pixel[3] = 0xFF;                         // A always opaque
                }

                pixels.render().unwrap();
                window.request_redraw();
            }

            _ => {}
        }
    });
}