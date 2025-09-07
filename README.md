# rust-tui-fm

**rust-tui-fm** is a fast, extensible, and cross-platform terminal file manager built with Rust. It features a three-column interface inspired by modern file managers, designed for efficiency and keyboard-driven navigation.

![Screenshot (placeholder)](placeholder.png)

## Features

*   **Three-Column Layout:**
    *   **Left Pane:** Quick access to XDG user folders, bookmarks, and mounted devices.
    *   **Middle Pane:** Main file list with support for sorting and filtering.
    *   **Right Pane:** Asynchronous preview for text files.
*   **Asynchronous Operations:** File operations (copy, move, delete) are handled in the background, keeping the UI responsive.
*   **Tabbed Interface:** Manage multiple directories with tabs.
*   **Extensible:** A plugin system (work in progress) allows for new functionality to be added.
*   **Configurable:** Keybindings and themes can be customized via a `config.toml` file.

## Building and Running

### Prerequisites

*   Rust toolchain (https://rustup.rs/)

### Building

To build the project, clone the repository and run:

```bash
cargo build --release
```

The binary will be located at `target/release/app`.

### Running

To run the file manager directly, use:

```bash
cargo run --release
```

## Keybindings

### Global
*   `q`: Quit the application
*   `Ctrl+n`: New tab
*   `Ctrl+w`: Close current tab
*   `Ctrl+Tab`: Next tab
*   `Ctrl+Shift+Tab`: Previous tab
*   `Ctrl+\``: Toggle terminal view in footer

### Navigation (Middle Pane)
*   `j` / `Arrow Down`: Move cursor down
*   `k` / `Arrow Up`: Move cursor up
*   `h` / `Arrow Left`: Navigate to parent directory
*   `l` / `Arrow Right` / `Enter`: Enter selected directory

### File Operations
*   `y`: Yank (copy) selected file/directory to clipboard
*   `d`: Cut selected file/directory to clipboard
*   `p`: Paste from clipboard (creates a copy/move task)
*   `m`: Bookmark the current directory

## Configuration

A configuration file can be created at `~/.config/rust-tui-fm/config.toml`.

Example `config.toml`:

```toml
# Bookmarks are stored as a map of name to path
[bookmarks]
dotfiles = "~/.dotfiles"
projects = "~/dev/projects"
```
