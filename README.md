# GUI Projects Workspace

A Rust workspace for multiple GUI applications built with the **egui** framework.

## Workspace Structure

This workspace uses Cargo's workspace feature to share dependencies across multiple GUI projects, reducing build times and ensuring consistency.

```
gui_projects/
├── Cargo.toml          # Workspace configuration
├── apps/               # GUI applications
│   ├── note_app/       # Rich text note editor
│   └── calc_app/       # Scientific programmer calculator
└── README.md
```

## Shared Dependencies

All projects in this workspace share the following dependencies (defined in workspace root):

- **eframe** 0.29 - Application framework for egui
- **egui** 0.29 - Immediate mode GUI library
- **egui_extras** 0.29 - Additional egui features
- **rfd** 0.14 - Native file dialogs

## Projects

### 1. Note App (`apps/note_app`)

A rich text note editor with formatting capabilities.

**Features:**

- Direct inline text editing
- Mouse text selection
- Text formatting: Bold, Italic, Bold+Italic, Regular
- Dynamic font sizing (8-72px)
- File open/save functionality
- Real-time formatting preview

**Run:**

```bash
cargo run --package note_app
```

**Build:**

```bash
cargo build --package note_app
```

### 2. Calculator App (`apps/calc_app`)

A comprehensive scientific and programmer calculator with extensive features.

**Features:**

- **18-digit precision** calculations
- **Scientific functions**: trigonometry (sin, cos, tan), logarithms, powers, roots, factorials
- **Memory operations**: M+, M-, MR, MC
- **Base conversion**: DEC, BIN, OCT, HEX
- **Bitwise operations**: NOT, AND, OR, XOR, NAND, NOR, XNOR
- **Bit manipulation**: shift left/right, rotate left/right
- **Programmer tools**: ASCII display, 2's complement, bit counting
- **Expression evaluator**: Type and evaluate complex mathematical expressions
- **Keyboard support**: Full numeric keypad and operator keys
- **Color-coded buttons**: Visual organization for different functions
- **Angle modes**: Degrees and Radians for trigonometric functions
- **1024×1024 window** with two-column layout

**Run:**

```bash
cargo run --package calc_app
```

**Build:**

```bash
cargo build --package calc_app
```

## Building the Workspace

Build all projects:

```bash
cargo build --workspace
```

Build in release mode:

```bash
cargo build --workspace --release
```

## Adding New Projects

1. Create a new project directory under `apps/`
2. Add the project to `members` in the root `Cargo.toml`
3. Use workspace dependencies with `dependency-name.workspace = true` in the project's `Cargo.toml`

Example:

```toml
[dependencies]
eframe.workspace = true
egui.workspace = true
```

## Development

All projects share the same build directory (`target/`) to maximize build cache efficiency.

**Clean build artifacts:**

```bash
cargo clean
```

**Check all projects:**

```bash
cargo check --workspace
```

## License

Individual projects may have their own licenses.
