# Rust SDL2 Basic Window

This project demonstrates a minimal SDL2 application in Rust that opens a black window.

## Prerequisites

Before you can build and run this project, you need to have Rust and Cargo installed. You can find installation instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

You also need to have the SDL2 development libraries installed on your system. The installation method varies depending on your operating system:

*   **Linux (Ubuntu/Debian):**
    ```bash
    sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev
    ```
*   **Linux (Fedora):**
    ```bash
    sudo dnf install SDL2-devel SDL2_image-devel SDL2_ttf-devel SDL2_mixer-devel
    ```
*   **macOS (using Homebrew):**
    ```bash
    brew install sdl2 sdl2_image sdl2_ttf sdl2_mixer
    ```
*   **Windows (using vcpkg or MSYS2):**
    Follow the instructions on the [rust-sdl2 crate page](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc-gnumingw) for setting up SDL2 on Windows. Typically, for MSVC, you might copy DLLs or use vcpkg. For GNU/MinGW, you'd place them in appropriate linker paths.

## Build

To build the project, navigate to the `rust_sdl2_window` directory in your terminal and run:

```bash
cargo build
```

## Run

After a successful build, you can run the application using:

```bash
cargo run
```

This will open a window with a black background. Press the Escape key or close the window to exit.
