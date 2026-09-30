// Derived from Smithay smallvil (MIT); see THIRD_PARTY.md.
#![allow(irrefutable_let_patterns)]

mod grabs;
mod handlers;
mod input;
mod state;
mod winit;

use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::{Display, DisplayHandle},
};
pub use state::Naruwm;

pub struct CalloopData {
    state: Naruwm,
    display_handle: DisplayHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "naruwm=info,smithay=warn".into()),
        )
        .init();

    // Parse before opening a display, so --help also works outside a desktop.
    let mut args = std::env::args_os().skip(1);
    let command: Vec<_> = match args.next().as_deref().and_then(|s| s.to_str()) {
        Some("--help" | "-h") => {
            println!("naruwm — nested Wayland compositor\n\nUsage: naruwm [-- COMMAND [ARGS...]]\n\nWith no command, starts with an empty desktop.\nRun with: naruwm -- weston-terminal\nClose the host window to exit. Click a client to focus it.\nClient-side title bars and borders support moving and resizing.");
            return Ok(());
        }
        Some("--") => {
            let command: Vec<_> = args.collect();
            if command.is_empty() {
                return Err("expected a command after --".into());
            }
            command
        }
        None => Vec::new(),
        _ => return Err("unknown argument; use naruwm --help".into()),
    };

    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        return Err("XDG_RUNTIME_DIR is not set; run naruwm inside a desktop session".into());
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
    let display: Display<Naruwm> = Display::new()?;
    let display_handle = display.handle();
    let state = Naruwm::new(&mut event_loop, display);
    let mut data = CalloopData {
        state,
        display_handle,
    };

    crate::winit::init_winit(&mut event_loop, &mut data)?;
    tracing::info!(socket = ?data.state.socket_name, "naruwm ready; connect clients using WAYLAND_DISPLAY");

    // Set the socket only for the child. The backend must retain the host display.
    let mut child = if let Some(program) = command.first() {
        Some(
            std::process::Command::new(program)
                .args(&command[1..])
                .env("WAYLAND_DISPLAY", &data.state.socket_name)
                .env_remove("WAYLAND_SOCKET")
                .env_remove("DISPLAY")
                .spawn()
                .map_err(|err| format!("failed to launch {program:?}: {err}"))?,
        )
    } else {
        None
    };

    let result = event_loop.run(None, &mut data, |_| {
        if let Some(process) = child.as_mut() {
            if matches!(process.try_wait(), Ok(Some(_))) {
                child = None;
            }
        }
    });
    // Reap the directly launched child on exit; its Wayland connection is closing.
    if let Some(mut process) = child {
        let _ = process.kill();
        let _ = process.wait();
    }
    result?;
    Ok(())
}
