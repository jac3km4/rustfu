# rustfu

`rustfu` is a Rust implementation of the ANM2 graphics engine.

## Overview

The repository consists of a cargo workspace with two primary members:
- `renderer`: The core rendering library for the ANM2 graphics engine.
- `gui`: The graphical user interface application built on top of the renderer.

## Technologies Used

- **Rust**: The primary programming language used for the project.
- **[Notan](https://github.com/Nazariglez/notan)**: Used as the core application and drawing framework.
- **[rfd](https://github.com/PolyMeilex/rfd)**: Used for native file dialogs.

## Getting Started

To get started, you can run the following commands from the project root:

- Run the GUI application:
  ```bash
  cargo run -p rustfu-gui
  ```
- Run compilation checks for the entire workspace (including `gui` and `renderer`):
  ```bash
  cargo check
  ```
- Execute the test suite for the entire workspace:
  ```bash
  cargo test
  ```
