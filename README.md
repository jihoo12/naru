# naru / naruwm

**naruwm** is the first component of the naru desktop environment.
It is a basic Wayland compositor built with Rust and Smithay 0.7,
running inside a window on an existing Wayland or X11 desktop.

## Development and running

Nix with flakes enabled is required. Run these commands in a graphical desktop session:

```sh
nix develop path:.
cargo run -- -- weston-terminal
```

`path:.` includes files that have not yet been added to Git.
Keep both `flake.lock` and `Cargo.lock` in version control to pin dependencies.
The first run takes additional time to download dependencies and compile the project.

- Click the terminal to give it keyboard focus.
- Move and resize windows using their client-side title bars and borders.
- Close the outer `naruwm — naru` window to exit the compositor.
- Run `cargo run` to start with an empty desktop.
- Run `cargo run -- --help` to view command-line options.
- Arguments after `--` specify an executable and its arguments, rather than a shell command string.

To connect additional applications from another terminal, use the socket name printed
in the logs. Enter the same development environment and replace the example socket
name below with the actual one:

```sh
WAYLAND_DISPLAY=wayland-1 weston-terminal
WAYLAND_DISPLAY=wayland-1 wayland-info
```

The compositor leaves the parent session's `WAYLAND_DISPLAY` unchanged.
It passes its socket only to the application it launches and cleans up that direct
child process when it exits.

## Validation and package builds

```sh
nix develop path:. -c cargo fmt --check
nix develop path:. -c cargo clippy --all-targets -- -D warnings
nix develop path:. -c cargo build
nix develop path:. -c python3 tools/smoke.py
nix build path:.
./result/bin/naruwm
```

The smoke test runs naruwm and a terminal inside a headless Weston instance,
checking actual buffer submission and frame callback delivery. Keyboard and mouse
interaction, along with visual correctness, still require manual verification.

On NixOS, naruwm uses the system graphics drivers. Other Linux distributions may
need a setup such as nixGL to connect host drivers with Nix OpenGL libraries.

## Current scope

- Wayland sockets and `wl_shm` client buffers
- `xdg-shell` windows and basic popup rendering
- Keyboard, pointer, scrolling, click-to-focus, and window stacking
- Client-requested window movement and resizing
- Nested output resizing and OpenGL ES compositing
- Focus-based clipboard data-device routing

Tiling, workspaces, global shortcuts, server-side decorations, full popup grabs,
application cursor images, XWayland, layer-shell, DMA-BUF, and standalone DRM/KMS
sessions are not implemented yet. The compositor currently uses the host cursor.
This is a foundation for further development, not yet a desktop environment for daily use.

## Code layout

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Command-line options, event loop, and application launch |
| `src/state.rs` | Wayland server and compositor state |
| `src/winit.rs` | Nested output and rendering |
| `src/input.rs` | Keyboard and pointer input, and focus |
| `src/handlers/` | Wayland protocol handlers |
| `src/grabs/` | Window movement and resizing |

The compositor is based on Smithay's smallvil example.
See [THIRD_PARTY.md](THIRD_PARTY.md) for attribution and licensing details.
