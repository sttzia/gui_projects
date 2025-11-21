# Scientific Programmer Calculator

A feature-rich calculator application built with Rust and egui, combining scientific computing capabilities with programmer-specific tools.

## Features

### Display

- **18-digit precision** for accurate calculations
- **900×150px display** with scrollable multi-line support
- Right-aligned output with text wrapping for large numbers
- Support for scientific notation for large/small numbers
- **Multiple Display Formats**:
  - **Regular**: Standard number format with automatic scientific notation for very large/small values
  - **Fixed**: Fixed 6 decimal places
  - **Scientific**: Always display in scientific notation (e.g., 1.234e8)
  - **Engineer**: Engineering notation with exponents as multiples of 3 (e.g., 123.456e6)
  - **Triads**: Thousands separators with commas (e.g., 123,456,789.123)
- **Error Display**: Shows overflow errors with previous valid result displayed below
- **Hover tooltips**: Scroll to see more for long results

### Basic Operations

- Addition (+), Subtraction (-), Multiplication (×), Division (÷)
- Decimal point support
- Clear (C) and Clear Entry (CE) functions
- Delete (DEL) - light blue colored button for single character removal

### Scientific Functions

- **Trigonometry**: sin, cos, tan, asin, acos, atan
- **Hyperbolic**: sinh, cosh, tanh
- **Logarithms**: log (base 10), ln (natural logarithm)
- **Powers & Roots**:
  - x² (square)
  - x^y (power)
  - √ (square root)
  - y-Root (nth root)
- **Special Functions**:
  - **n!**: Standard factorial (up to 170!) using f64
  - **n!!**: Large factorial (up to 100,000!) using arbitrary precision (gold button)
  - Reciprocal (1/x)
  - Absolute value (Abs)
- **Constants**: π (pi), e (Euler's number)
- **Modulo** operation (%)

### Memory Functions

- **M+**: Add current value to memory
- **M-**: Subtract current value from memory
- **MR**: Recall memory value
- **MC**: Clear memory

### Number Base Conversion

- **DEC**: Decimal (base 10)
- **BIN**: Binary (base 2)
- **OCT**: Octal (base 8)
- **HEX**: Hexadecimal (base 16)

### Bitwise Operations

Operates on 64-bit integers:

- **NOT**: Bitwise complement (~)
- **AND**: Bitwise AND (&)
- **OR**: Bitwise OR (|)
- **XOR**: Bitwise XOR (^)
- **NAND**: Bitwise NAND
- **NOR**: Bitwise NOR
- **XNOR**: Bitwise XNOR

### Bit Manipulation

- **<< (Shift Left)**: Left shift by 1 bit
- **>> (Shift Right)**: Right shift by 1 bit
- **ROR (Rotate Right)**: Circular right rotation by 1 bit
- **ROL (Rotate Left)**: Circular left rotation by 1 bit

### Programmer Tools

- **ASCII**: Display ASCII character for values 0-127
- **2's Comp**: Two's complement (negation)
- **BitCount**: Count number of set bits (1s)

### Expression Evaluator

Type mathematical expressions directly in the input field and press Enter or click "Eval":

- Supports: `+`, `-`, `*`, `/`, `^` (power), `%` (modulo)
- Functions: `sqrt()`, `sin()`, `cos()`, `tan()`, `log()`, `ln()`, `factorial()` or `fact()`
- Probability: `nPr(n,r)` (permutations), `nCr(n,r)` (combinations)
- Constants: `pi`, `e`
- Parentheses for grouping
- Examples:
  - `2 * (3 + sqrt(16)) / pi`
  - `factorial(5)` or `fact(5)` → 120
  - `nCr(49,6)` → 13983816 (lottery combinations)
  - `15 % 7` → 1 (modulo operation)

### Statistics Functions

- **Data Entry**: Add values to statistical dataset
- **Mean**: Calculate average of all data points
- **Sum**: Calculate total sum of all data points
- **Count**: Display number of data points
- **Std Dev**: Calculate standard deviation
- **Variance**: Calculate variance
- **Clear Data**: Reset the statistical dataset
- **View Data**: Display all entered data in a scrollable window

### Probability Functions

- **nPr (Permutations)**: Calculate number of permutations (order matters)
  - Button: Select n, press nPr, select r, press =
  - Expression: `nPr(49,6)` → 10,068,347,520
- **nCr (Combinations)**: Calculate number of combinations (order doesn't matter)
  - Button: Select n, press nCr, select r, press =
  - Expression: `nCr(49,6)` → 13,983,816
- Maximum n value: 170 (to prevent overflow)

### Large Number Factorial

- **n!! Button** (gold colored): Calculate factorials beyond f64 limits
  - Uses arbitrary precision arithmetic (BigUint)
  - Can calculate factorials up to 100,000!
  - Results displayed with thousands separators
  - Automatically wraps across multiple lines
  - Examples:
    - 200! = 788,657,867,364,790,503... (375 digits)
    - 1000! (2,568 digits)
  - **Why factorial limit?**: Standard f64 can only represent up to 170! (≈7.257×10³⁰⁶). Beyond this, the n!! button uses BigUint to handle arbitrarily large integers.

### Angle Mode

- **DEG**: Degrees mode for trigonometric functions
- **RAD**: Radians mode for trigonometric functions

## Usage

### Running the Application

```bash
cargo run --package calc_app
```

### Keyboard Shortcuts

- **Numeric Keys (0-9)**: Enter digits
- **Numeric Keypad (Num0-Num9)**: Enter digits
- **+ (Plus)**: Addition
- **- (Minus)**: Subtraction
- **× or \* (Multiply)**: Multiplication
- **÷ or / (Divide)**: Division
- **. (Decimal)**: Decimal point
- **Enter**: Calculate result (equals)
- **Escape**: Clear display
- **Backspace**: Delete last character

### Mouse Controls

Click any button to perform the corresponding operation.

## Window Configuration

- **Dimensions**: 1024×768 pixels
- **Layout**: Two-column design
  - Left column: Standard calculator functions
  - Right column: Programmer-specific tools
- **Left Margin**: 1cm (37.8px) for visual balance

## Technical Details

- **Framework**: egui 0.29 with eframe
- **Language**: Rust
- **Precision**: 64-bit floating-point (f64) for calculations
- **Integer Operations**: 64-bit signed integers (i64) for bitwise operations
- **Color Coding**:
  - Green button: Equals (=)
  - Orange button: Clear (C)
  - Red button: Clear Entry (CE)
  - Light Blue button: Delete (DEL)
  - Gold button: Large factorial (n!!)

## Building from Source

```bash
cd apps/calc_app
cargo build --release
```

## Dependencies

- `eframe` 0.29: Application framework
- `egui` 0.29: Immediate mode GUI library
- `num-bigint` 0.4: Arbitrary precision integer arithmetic for large factorials
- `num-traits` 0.2: Numeric traits for big integer operations

## License

This project is part of the gui_projects workspace.
