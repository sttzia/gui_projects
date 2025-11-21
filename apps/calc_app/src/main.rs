use eframe::egui;
use egui::{Color32, RichText, Vec2};
use num_bigint::BigUint;
use num_traits::One;
use std::f64::consts::{E, PI};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 1024.0])
            .with_title("Scientific Calculator"),
        ..Default::default()
    };
    eframe::run_native(
        "Scientific Calculator",
        options,
        Box::new(|_cc| Ok(Box::<Calculator>::default())),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Root,
    Modulo,
    Permutation, // nPr
    Combination, // nCr
}

#[derive(Clone, Copy, PartialEq)]
enum DisplayFormat {
    Regular,     // Standard format
    Fixed,       // Fixed decimal places
    Scientific,  // Scientific notation
    Engineering, // Engineering notation (exponent is multiple of 3)
    Triads,      // Thousands separators (commas)
}

struct Calculator {
    display: String,
    current_value: f64,
    operation: Option<Operation>,
    new_number: bool,
    memory: f64,
    degree_mode: bool, // true = degrees, false = radians
    expression_input: String,
    base_mode: String, // "DEC", "BIN", "OCT", "HEX"
    bitwise_operand: Option<i64>,
    stat_data: Vec<f64>,           // Data for statistics calculations
    previous_display: String,      // Store previous value before overflow
    display_format: DisplayFormat, // Number display format
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: "0".to_string(),
            current_value: 0.0,
            operation: None,
            new_number: true,
            memory: 0.0,
            degree_mode: true,
            expression_input: String::new(),
            base_mode: "DEC".to_string(),
            bitwise_operand: None,
            stat_data: Vec::new(),
            previous_display: String::new(),
            display_format: DisplayFormat::Regular,
        }
    }
}

impl Calculator {
    fn set_display_result(&mut self, num: f64) {
        if num.is_infinite() || num.is_nan() {
            // Store the previous display value before showing error
            if !self.display.starts_with("Error:") {
                self.previous_display = self.display.clone();
            }
            self.display = if num.is_infinite() {
                "Error: Overflow".to_string()
            } else {
                "Error: Invalid".to_string()
            };
        } else {
            self.previous_display.clear();
            self.display = self.format_number_with_style(num);
        }
    }

    fn format_number_with_style(&self, num: f64) -> String {
        if num.is_infinite() {
            return "Error: Overflow".to_string();
        }
        if num.is_nan() {
            return "Error: Invalid".to_string();
        }

        match self.display_format {
            DisplayFormat::Regular => {
                // Standard format with up to 18 significant digits
                let formatted = format!("{:.18}", num);
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                if num.abs() >= 1e15 || (num.abs() < 1e-15 && num != 0.0) {
                    format!("{:.12e}", num)
                } else {
                    trimmed.to_string()
                }
            }
            DisplayFormat::Fixed => {
                // Fixed 6 decimal places
                format!("{:.6}", num)
            }
            DisplayFormat::Scientific => {
                // Always scientific notation
                format!("{:.12e}", num)
            }
            DisplayFormat::Engineering => {
                // Engineering notation (exponent is multiple of 3)
                if num == 0.0 {
                    return "0.000000000000e0".to_string();
                }

                let abs_num = num.abs();
                let sign = if num < 0.0 { "-" } else { "" };

                // Calculate the base-10 exponent
                let exponent = abs_num.log10().floor() as i32;

                // Round down to nearest multiple of 3
                let eng_exponent = (exponent / 3) * 3;

                // Calculate mantissa (should be between 1 and 999.999...)
                let mantissa = abs_num / 10_f64.powi(eng_exponent);

                format!(
                    "{}{}e{}",
                    sign,
                    format!("{:.9}", mantissa)
                        .trim_end_matches('0')
                        .trim_end_matches('.'),
                    eng_exponent
                )
            }
            DisplayFormat::Triads => {
                // Format with thousands separators
                let formatted = format!("{:.18}", num);
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

                if let Some(dot_pos) = trimmed.find('.') {
                    let integer_part = &trimmed[..dot_pos];
                    let decimal_part = &trimmed[dot_pos..];
                    let formatted_int = self.add_thousands_separators(integer_part);
                    format!("{}{}", formatted_int, decimal_part)
                } else {
                    self.add_thousands_separators(trimmed)
                }
            }
        }
    }

    fn add_thousands_separators(&self, num_str: &str) -> String {
        let is_negative = num_str.starts_with('-');
        let num_str = if is_negative { &num_str[1..] } else { num_str };

        let len = num_str.len();
        if len <= 3 {
            return if is_negative {
                format!("-{}", num_str)
            } else {
                num_str.to_string()
            };
        }

        let mut formatted = String::new();
        for (i, ch) in num_str.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                formatted.push(',');
            }
            formatted.push(ch);
        }

        if is_negative {
            format!("-{}", formatted)
        } else {
            formatted
        }
    }

    fn append_digit(&mut self, digit: &str) {
        if self.new_number {
            self.display = digit.to_string();
            self.new_number = false;
        } else {
            if self.display == "0" && digit != "." {
                self.display = digit.to_string();
            } else if !(digit == "." && self.display.contains('.')) {
                // Limit to 18 digits precision (not counting decimal point)
                let digit_count = self.display.chars().filter(|c| c.is_numeric()).count();
                if digit_count < 18 {
                    self.display.push_str(digit);
                }
            }
        }
    }

    fn clear(&mut self) {
        self.display = "0".to_string();
        self.current_value = 0.0;
        self.operation = None;
        self.new_number = true;
    }

    fn clear_entry(&mut self) {
        self.display = "0".to_string();
        self.new_number = true;
    }

    fn set_operation(&mut self, op: Operation) {
        if !self.new_number {
            self.calculate();
        }
        self.current_value = self.get_display_value();
        self.operation = Some(op);
        self.new_number = true;
    }

    fn calculate(&mut self) {
        if let Some(op) = self.operation {
            let second = self.get_display_value();
            let result = match op {
                Operation::Add => self.current_value + second,
                Operation::Subtract => self.current_value - second,
                Operation::Multiply => self.current_value * second,
                Operation::Divide => {
                    if second != 0.0 {
                        self.current_value / second
                    } else {
                        self.display = "Error: Div by 0".to_string();
                        self.new_number = true;
                        return;
                    }
                }
                Operation::Power => self.current_value.powf(second),
                Operation::Root => {
                    if second != 0.0 {
                        self.current_value.powf(1.0 / second)
                    } else {
                        self.display = "Error: Root 0".to_string();
                        self.new_number = true;
                        return;
                    }
                }
                Operation::Modulo => self.current_value % second,
                Operation::Permutation => {
                    self.permutation(self.current_value, second);
                    return;
                }
                Operation::Combination => {
                    self.combination(self.current_value, second);
                    return;
                }
            };
            self.set_display_result(result);
            self.current_value = result;
            self.operation = None;
            self.new_number = true;
        }
    }

    fn get_display_value(&self) -> f64 {
        // Parse display value according to current base mode
        match self.base_mode.as_str() {
            "BIN" => i64::from_str_radix(&self.display, 2).unwrap_or(0) as f64,
            "OCT" => i64::from_str_radix(&self.display, 8).unwrap_or(0) as f64,
            "HEX" => i64::from_str_radix(&self.display, 16).unwrap_or(0) as f64,
            _ => self.display.parse().unwrap_or(0.0), // DEC
        }
    }

    fn apply_function<F>(&mut self, f: F)
    where
        F: Fn(f64) -> f64,
    {
        let value = self.get_display_value();
        let result = f(value);
        self.set_display_result(result);
        self.new_number = true;
    }

    fn evaluate_expression(&mut self) {
        let expr = self.expression_input.trim();
        if expr.is_empty() {
            return;
        }

        // Simple expression evaluator
        match self.parse_and_evaluate(expr) {
            Ok(result) => {
                self.set_display_result(result);
                self.new_number = true;
                self.expression_input.clear();
            }
            Err(e) => {
                self.display = format!("Error: {}", e);
                self.new_number = true;
            }
        }
    }

    fn parse_and_evaluate(&self, expr: &str) -> Result<f64, String> {
        // Remove spaces
        let mut expr = expr.replace(" ", "");

        // Handle implicit multiplication: )( -> )*(
        expr = expr.replace(")(", ")*(");
        // Handle implicit multiplication: number( -> number*(
        expr = self.add_implicit_multiplication(&expr);

        // Try to evaluate as a simple arithmetic expression
        self.evaluate_with_precedence(&expr)
    }

    fn add_implicit_multiplication(&self, expr: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = expr.chars().collect();

        for i in 0..chars.len() {
            result.push(chars[i]);

            if i + 1 < chars.len() {
                let current = chars[i];
                let next = chars[i + 1];

                // Add * between: digit and (, ) and digit, ) and (
                if (current.is_numeric() && next == '(')
                    || (current == ')' && next.is_numeric())
                    || (current == ')' && next == '(')
                {
                    result.push('*');
                }
            }
        }
        result
    }

    fn evaluate_with_precedence(&self, expr: &str) -> Result<f64, String> {
        // Handle parentheses first
        if let Some(result) = self.handle_parentheses(expr)? {
            return Ok(result);
        }

        // Check for addition/subtraction (lowest precedence)
        // Need to skip operators inside parentheses
        if let Some(pos) = self.find_operator_outside_parens(expr, '+') {
            let left = self.evaluate_with_precedence(&expr[..pos])?;
            let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
            return Ok(left + right);
        }

        if let Some(pos) = self.find_operator_outside_parens(expr, '-') {
            if pos > 0 {
                // Check if it's a negative sign or subtraction
                let prev_char = expr.chars().nth(pos - 1);
                if let Some(ch) = prev_char {
                    if ch != '('
                        && ch != '*'
                        && ch != '/'
                        && ch != '^'
                        && ch != '+'
                        && ch != '-'
                        && ch != '%'
                    {
                        let left = self.evaluate_with_precedence(&expr[..pos])?;
                        let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
                        return Ok(left - right);
                    }
                }
            }
        }

        // Check for multiplication/division
        if let Some(pos) = self.find_operator_outside_parens(expr, '*') {
            let left = self.evaluate_with_precedence(&expr[..pos])?;
            let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
            return Ok(left * right);
        }

        if let Some(pos) = self.find_operator_outside_parens(expr, '/') {
            let left = self.evaluate_with_precedence(&expr[..pos])?;
            let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
            if right == 0.0 {
                return Err("Division by zero".to_string());
            }
            return Ok(left / right);
        }

        // Check for modulo
        if let Some(pos) = self.find_operator_outside_parens(expr, '%') {
            let left = self.evaluate_with_precedence(&expr[..pos])?;
            let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
            return Ok(left % right);
        }

        // Check for power
        if let Some(pos) = self.find_operator_outside_parens(expr, '^') {
            let left = self.evaluate_with_precedence(&expr[..pos])?;
            let right = self.evaluate_with_precedence(&expr[pos + 1..])?;
            return Ok(left.powf(right));
        }

        // Handle functions
        if expr.starts_with("sqrt(") && expr.ends_with(")") {
            let inner = &expr[5..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            return Ok(val.sqrt());
        }

        if expr.starts_with("sin(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            let angle = if self.degree_mode {
                val * PI / 180.0
            } else {
                val
            };
            return Ok(angle.sin());
        }

        if expr.starts_with("cos(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            let angle = if self.degree_mode {
                val * PI / 180.0
            } else {
                val
            };
            return Ok(angle.cos());
        }

        if expr.starts_with("tan(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            let angle = if self.degree_mode {
                val * PI / 180.0
            } else {
                val
            };
            return Ok(angle.tan());
        }

        if expr.starts_with("log(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            return Ok(val.log10());
        }

        if expr.starts_with("ln(") && expr.ends_with(")") {
            let inner = &expr[3..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            return Ok(val.ln());
        }

        // Handle factorial function
        if expr.starts_with("factorial(") && expr.ends_with(")") {
            let inner = &expr[10..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            return Ok(self.factorial(val));
        }

        if expr.starts_with("fact(") && expr.ends_with(")") {
            let inner = &expr[5..expr.len() - 1];
            let val = self.evaluate_with_precedence(inner)?;
            return Ok(self.factorial(val));
        }

        // Handle nPr and nCr functions
        if expr.starts_with("nPr(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let n = self.evaluate_with_precedence(&inner[..comma_pos])?;
                let r = self.evaluate_with_precedence(&inner[comma_pos + 1..])?;
                if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
                    return Err("Invalid nPr arguments".to_string());
                }
                if n > 170.0 {
                    return Err("n too large (max 170)".to_string());
                }
                // Calculate nPr efficiently without overflow
                let mut result = 1.0_f64;
                for i in 0..(r as i32) {
                    result *= n - i as f64;
                }
                return Ok(result);
            }
            return Err("nPr requires two arguments: nPr(n,r)".to_string());
        }

        if expr.starts_with("nCr(") && expr.ends_with(")") {
            let inner = &expr[4..expr.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let n = self.evaluate_with_precedence(&inner[..comma_pos])?;
                let r = self.evaluate_with_precedence(&inner[comma_pos + 1..])?;
                if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
                    return Err("Invalid nCr arguments".to_string());
                }
                if n > 170.0 {
                    return Err("n too large (max 170)".to_string());
                }
                // Calculate nCr efficiently without overflow
                let mut result = 1.0_f64;
                let r_use = if r > n - r { n - r } else { r };
                for i in 0..(r_use as i32) {
                    result *= (n - i as f64) / (i as f64 + 1.0);
                }
                return Ok(result);
            }
            return Err("nCr requires two arguments: nCr(n,r)".to_string());
        }

        // Handle parentheses
        if expr.starts_with("(") && expr.ends_with(")") {
            return self.evaluate_with_precedence(&expr[1..expr.len() - 1]);
        }

        // Handle constants
        if expr == "pi" {
            return Ok(PI);
        }
        if expr == "e" {
            return Ok(E);
        }

        // Try to parse as a number
        expr.parse::<f64>()
            .map_err(|_| format!("Invalid expression: {}", expr))
    }

    // Find the rightmost occurrence of an operator outside of parentheses
    fn find_operator_outside_parens(&self, expr: &str, op: char) -> Option<usize> {
        let mut depth = 0;
        let mut last_pos = None;

        for (i, c) in expr.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {
                    if c == op && depth == 0 {
                        last_pos = Some(i);
                    }
                }
            }
        }

        last_pos
    }

    // Handle parentheses - check if entire expression is wrapped
    fn handle_parentheses(&self, expr: &str) -> Result<Option<f64>, String> {
        if expr.starts_with("(") && expr.ends_with(")") {
            // Verify matching parentheses
            let mut depth = 0;
            for (i, c) in expr.chars().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                // If depth reaches 0 before the end, outer parens don't wrap everything
                if depth == 0 && i < expr.len() - 1 {
                    return Ok(None);
                }
            }
            // The entire expression is wrapped in parentheses
            return Ok(Some(
                self.evaluate_with_precedence(&expr[1..expr.len() - 1])?,
            ));
        }
        Ok(None)
    }

    fn convert_base(&mut self, new_base: &str) {
        // Get the numeric value from current base
        let current_val = self.get_display_value() as i64;

        // Update base mode
        self.base_mode = new_base.to_string();

        // Format display in new base
        self.display = match new_base {
            "BIN" => format!("{:b}", current_val),
            "OCT" => format!("{:o}", current_val),
            "HEX" => format!("{:X}", current_val),
            _ => current_val.to_string(), // DEC
        };
        self.new_number = true;
    }

    fn apply_bitwise_not(&mut self) {
        let val = self.get_display_value() as i64;
        let result = !val;
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    fn set_bitwise_operation(&mut self, op: &str) {
        let val = self.get_display_value() as i64;
        self.bitwise_operand = Some(val);
        self.display = op.to_string();
        self.new_number = true;
    }

    fn apply_shift_left(&mut self) {
        let val = self.get_display_value() as i64;
        let result = val << 1; // Shift left by 1 bit
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    fn apply_shift_right(&mut self) {
        let val = self.get_display_value() as i64;
        let result = val >> 1; // Shift right by 1 bit
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    fn show_ascii_value(&mut self) {
        let val = self.get_display_value() as u8;
        if val < 128 {
            let ch = val as char;
            self.display = format!("{} = '{}'", val, ch);
        } else {
            self.display = format!("{} (non-ASCII)", val);
        }
        self.new_number = true;
    }

    fn apply_twos_complement(&mut self) {
        let val = self.get_display_value() as i64;
        let result = -val; // Two's complement is simply negation in Rust
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    fn count_bits(&mut self) {
        let val = self.get_display_value() as u64;
        let count = val.count_ones(); // Count set bits (1s)
        self.display = format!("{} bits set", count);
        self.new_number = true;
    }

    fn apply_rotate_left(&mut self) {
        let val = self.get_display_value() as u32;
        let result = val.rotate_left(1); // Rotate left by 1 bit
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    fn apply_rotate_right(&mut self) {
        let val = self.get_display_value() as u32;
        let result = val.rotate_right(1); // Rotate right by 1 bit
        self.display = format_number(result as f64);
        self.new_number = true;
    }

    // Statistics Functions
    fn stat_add_data(&mut self) {
        let value = self.get_display_value();
        self.stat_data.push(value);
        self.display = format!("Data: {} items", self.stat_data.len());
        self.new_number = true;
    }

    fn stat_clear(&mut self) {
        self.stat_data.clear();
        self.display = "Data cleared".to_string();
        self.new_number = true;
    }

    fn stat_mean(&mut self) {
        if self.stat_data.is_empty() {
            self.display = "Error: No data".to_string();
        } else {
            let sum: f64 = self.stat_data.iter().sum();
            let mean = sum / self.stat_data.len() as f64;
            self.display = format_number(mean);
        }
        self.new_number = true;
    }

    fn stat_sum(&mut self) {
        if self.stat_data.is_empty() {
            self.display = "Error: No data".to_string();
        } else {
            let sum: f64 = self.stat_data.iter().sum();
            self.display = format_number(sum);
        }
        self.new_number = true;
    }

    fn stat_count(&mut self) {
        self.display = format!("{}", self.stat_data.len());
        self.new_number = true;
    }

    fn stat_std_dev(&mut self) {
        if self.stat_data.len() < 2 {
            self.display = "Error: Need 2+ values".to_string();
        } else {
            let mean = self.stat_data.iter().sum::<f64>() / self.stat_data.len() as f64;
            let variance = self
                .stat_data
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>()
                / self.stat_data.len() as f64;
            let std_dev = variance.sqrt();
            self.display = format_number(std_dev);
        }
        self.new_number = true;
    }

    fn stat_variance(&mut self) {
        if self.stat_data.len() < 2 {
            self.display = "Error: Need 2+ values".to_string();
        } else {
            let mean = self.stat_data.iter().sum::<f64>() / self.stat_data.len() as f64;
            let variance = self
                .stat_data
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>()
                / self.stat_data.len() as f64;
            self.display = format_number(variance);
        }
        self.new_number = true;
    }

    // Probability Functions
    fn permutation(&mut self, n: f64, r: f64) {
        if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
            self.display = "Error: Invalid nPr".to_string();
        } else if n > 170.0 {
            self.display = "Error: n too large".to_string();
        } else {
            // Calculate nPr = n! / (n-r)! more efficiently
            let mut result = 1.0_f64;
            for i in 0..(r as i32) {
                result *= n - i as f64;
            }
            self.display = format_number(result);
        }
        self.new_number = true;
    }

    fn combination(&mut self, n: f64, r: f64) {
        if n < 0.0 || r < 0.0 || r > n || n.fract() != 0.0 || r.fract() != 0.0 {
            self.display = "Error: Invalid nCr".to_string();
        } else if n > 170.0 {
            self.display = "Error: n too large".to_string();
        } else {
            // Calculate nCr = n! / (r! * (n-r)!) more efficiently
            // nCr = (n * (n-1) * ... * (n-r+1)) / (r * (r-1) * ... * 1)
            let mut result = 1.0_f64;
            let r_use = if r > n - r { n - r } else { r }; // Use smaller of r and n-r
            for i in 0..(r_use as i32) {
                result *= (n - i as f64) / (i as f64 + 1.0);
            }
            self.display = format_number(result);
        }
        self.new_number = true;
    }

    // Calculate factorial using f64 to handle large values (up to ~170)
    fn factorial(&self, n: f64) -> f64 {
        if n < 0.0 || n.fract() != 0.0 {
            return f64::NAN; // Factorial only defined for non-negative integers
        }
        if n > 170.0 {
            return f64::INFINITY; // Overflow protection
        }
        let mut result = 1.0;
        for i in 2..=(n as i64) {
            result *= i as f64;
        }
        result
    }

    // Calculate large factorials using BigUint (for values > 170)
    fn big_factorial(&self, n: f64) -> String {
        if n < 0.0 || n.fract() != 0.0 {
            return "Error: Invalid (not a non-negative integer)".to_string();
        }
        if n > 100000.0 {
            return "Error: Too large (max 100000)".to_string();
        }

        let n_int = n as u64;
        let mut result: BigUint = One::one();

        for i in 2..=n_int {
            result *= i;
        }

        // Format with thousands separators for readability
        let result_str = result.to_string();
        self.format_with_separators(&result_str)
    }

    fn format_with_separators(&self, num_str: &str) -> String {
        let len = num_str.len();
        if len <= 3 {
            return num_str.to_string();
        }

        let mut formatted = String::new();
        for (i, ch) in num_str.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                formatted.push(',');
            }
            formatted.push(ch);
        }
        formatted
    }
}

fn format_number(num: f64) -> String {
    if num.is_infinite() {
        return "Error: Overflow".to_string();
    }
    if num.is_nan() {
        return "Error: Invalid".to_string();
    }

    // Format with up to 18 significant digits
    let formatted = format!("{:.18}", num);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

    // Handle very large or very small numbers with scientific notation
    if num.abs() >= 1e15 || (num.abs() < 1e-15 && num != 0.0) {
        format!("{:.12e}", num)
    } else {
        trimmed.to_string()
    }
}

impl eframe::App for Calculator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Handle keyboard input
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        repeat: false,
                        ..
                    } = event
                    {
                        // Ignore number keys when Shift is pressed (for parentheses and other symbols)
                        match key {
                            egui::Key::Num0 if !modifiers.shift => self.append_digit("0"),
                            egui::Key::Num1 if !modifiers.shift => self.append_digit("1"),
                            egui::Key::Num2 if !modifiers.shift => self.append_digit("2"),
                            egui::Key::Num3 if !modifiers.shift => self.append_digit("3"),
                            egui::Key::Num4 if !modifiers.shift => self.append_digit("4"),
                            egui::Key::Num5 if !modifiers.shift => self.append_digit("5"),
                            egui::Key::Num6 if !modifiers.shift => self.append_digit("6"),
                            egui::Key::Num7 if !modifiers.shift => self.append_digit("7"),
                            egui::Key::Num8 if !modifiers.shift => self.append_digit("8"),
                            egui::Key::Num9 if !modifiers.shift => self.append_digit("9"),
                            egui::Key::Plus => self.set_operation(Operation::Add),
                            egui::Key::Minus => self.set_operation(Operation::Subtract),
                            egui::Key::Enter => self.calculate(),
                            egui::Key::Escape => self.clear(),
                            egui::Key::Backspace => {
                                if !self.new_number && self.display.len() > 1 {
                                    self.display.pop();
                                } else {
                                    self.display = "0".to_string();
                                    self.new_number = true;
                                }
                            }
                            _ => {}
                        }
                    } else if let egui::Event::Text(text) = event {
                        // Handle text input for operators and decimal
                        match text.as_str() {
                            "+" => self.set_operation(Operation::Add),
                            "-" => self.set_operation(Operation::Subtract),
                            "*" => self.set_operation(Operation::Multiply),
                            "/" => self.set_operation(Operation::Divide),
                            "." => self.append_digit("."),
                            _ => {}
                        }
                    }
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Top margin
                ui.add_space(10.0);

                // Left margin (1cm ≈ 37.8 pixels at 96 DPI)
                ui.horizontal(|ui| {
                    ui.allocate_space(Vec2::new(37.8, 0.0));

                    ui.vertical(|ui| {
                        // Display at the top
                        egui::Frame::none()
                            .fill(Color32::from_gray(240))
                            .stroke(egui::Stroke::new(2.0, Color32::from_gray(100)))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_min_width(900.0);
                                ui.set_max_width(900.0);
                                ui.set_min_height(150.0);

                                // Check if we have an error with a previous value
                                if self.display.starts_with("Error:")
                                    && !self.previous_display.is_empty()
                                {
                                    ui.vertical(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let error_text = RichText::new(&self.display)
                                                    .size(32.0)
                                                    .monospace();
                                                ui.label(error_text);
                                            },
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let prev_text =
                                                    RichText::new(&self.previous_display)
                                                        .size(16.0)
                                                        .monospace()
                                                        .color(Color32::from_gray(120));
                                                ui.label(prev_text);
                                            },
                                        );
                                    });
                                } else {
                                    // Use ScrollArea for long numbers with text wrapping
                                    egui::ScrollArea::vertical()
                                        .max_height(130.0)
                                        .show(ui, |ui| {
                                            ui.with_layout(
                                                egui::Layout::top_down(egui::Align::Max),
                                                |ui| {
                                                    ui.set_max_width(880.0);
                                                    ui.add(
                                                        egui::Label::new(
                                                            RichText::new(&self.display)
                                                                .size(18.0)
                                                                .monospace(),
                                                        )
                                                        .wrap(),
                                                    );
                                                },
                                            );
                                        });
                                }
                            });

                        ui.add_space(10.0);

                        // Mode and Memory indicators
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Mode: {}",
                                if self.degree_mode { "DEG" } else { "RAD" }
                            ));
                            ui.separator();
                            ui.label(format!("Memory: {:.2}", self.memory));
                        });

                        ui.add_space(5.0);

                        // Display Format buttons
                        ui.horizontal(|ui| {
                            ui.label("Format:");
                            if ui.button("Regular").clicked() {
                                self.display_format = DisplayFormat::Regular;
                                if let Ok(val) = self.display.replace(",", "").parse::<f64>() {
                                    self.display = self.format_number_with_style(val);
                                }
                            }
                            if ui.button("Fixed").clicked() {
                                self.display_format = DisplayFormat::Fixed;
                                if let Ok(val) = self.display.replace(",", "").parse::<f64>() {
                                    self.display = self.format_number_with_style(val);
                                }
                            }
                            if ui.button("Scientific").clicked() {
                                self.display_format = DisplayFormat::Scientific;
                                if let Ok(val) = self.display.replace(",", "").parse::<f64>() {
                                    self.display = self.format_number_with_style(val);
                                }
                            }
                            if ui.button("Engineer").clicked() {
                                self.display_format = DisplayFormat::Engineering;
                                if let Ok(val) = self.display.replace(",", "").parse::<f64>() {
                                    self.display = self.format_number_with_style(val);
                                }
                            }
                            if ui.button("Triads").clicked() {
                                self.display_format = DisplayFormat::Triads;
                                if let Ok(val) = self.display.replace(",", "").parse::<f64>() {
                                    self.display = self.format_number_with_style(val);
                                }
                            }
                        });

                        ui.add_space(10.0);

                        // Main content area with buttons side by side
                        ui.horizontal(|ui| {
                            // Left column: All main calculator buttons
                            ui.vertical(|ui| {
                                // All buttons below the display
                                let button_size = Vec2::new(80.0, 40.0);
                                let small_button_size = Vec2::new(55.0, 40.0);

                                // Memory and Mode buttons
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("MC"))
                                        .clicked()
                                    {
                                        self.memory = 0.0;
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("MR"))
                                        .clicked()
                                    {
                                        self.display = format_number(self.memory);
                                        self.new_number = true;
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("M+"))
                                        .clicked()
                                    {
                                        self.memory += self.get_display_value();
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("M-"))
                                        .clicked()
                                    {
                                        self.memory -= self.get_display_value();
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("DEG/RAD"))
                                        .clicked()
                                    {
                                        self.degree_mode = !self.degree_mode;
                                    }
                                });

                                ui.add_space(5.0);

                                // Scientific functions row 1
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("sin"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            if deg_mode {
                                                (x * PI / 180.0).sin()
                                            } else {
                                                x.sin()
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("cos"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            if deg_mode {
                                                (x * PI / 180.0).cos()
                                            } else {
                                                x.cos()
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("tan"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            if deg_mode {
                                                (x * PI / 180.0).tan()
                                            } else {
                                                x.tan()
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("ln"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.ln());
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("log"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.log10());
                                    }
                                });

                                // Factorial row
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("n!"))
                                        .clicked()
                                    {
                                        let value = self.get_display_value();
                                        let result = self.factorial(value);
                                        self.set_display_result(result);
                                        self.new_number = true;
                                    }
                                    if ui
                                        .add_sized(
                                            small_button_size,
                                            egui::Button::new("n!!")
                                                .fill(Color32::from_rgb(255, 215, 0)),
                                        )
                                        .clicked()
                                    {
                                        let value = self.get_display_value();
                                        self.display = self.big_factorial(value);
                                        self.previous_display.clear();
                                        self.new_number = true;
                                    }
                                });

                                // Scientific functions row 2
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("asin"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            let result = x.asin();
                                            if deg_mode {
                                                result * 180.0 / PI
                                            } else {
                                                result
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("acos"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            let result = x.acos();
                                            if deg_mode {
                                                result * 180.0 / PI
                                            } else {
                                                result
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("atan"))
                                        .clicked()
                                    {
                                        let deg_mode = self.degree_mode;
                                        self.apply_function(|x| {
                                            let result = x.atan();
                                            if deg_mode {
                                                result * 180.0 / PI
                                            } else {
                                                result
                                            }
                                        });
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("√"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.sqrt());
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("x²"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x * x);
                                    }
                                });

                                // Scientific functions row 3
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("sinh"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.sinh());
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("cosh"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.cosh());
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("tanh"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| x.tanh());
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("π"))
                                        .clicked()
                                    {
                                        self.display = format_number(PI);
                                        self.new_number = true;
                                    }
                                    if ui
                                        .add_sized(small_button_size, egui::Button::new("e"))
                                        .clicked()
                                    {
                                        self.display = format_number(E);
                                        self.new_number = true;
                                    }
                                });

                                ui.add_space(5.0);

                                // Clear buttons
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(
                                            button_size,
                                            egui::Button::new("C")
                                                .fill(Color32::from_rgb(255, 165, 0)),
                                        )
                                        .clicked()
                                    {
                                        self.clear();
                                    }
                                    if ui
                                        .add_sized(
                                            button_size,
                                            egui::Button::new("CE")
                                                .fill(Color32::from_rgb(255, 0, 0)),
                                        )
                                        .clicked()
                                    {
                                        self.clear_entry();
                                    }
                                    if ui
                                        .add_sized(
                                            button_size,
                                            egui::Button::new("DEL")
                                                .fill(Color32::from_rgb(173, 216, 230)),
                                        )
                                        .clicked()
                                    {
                                        if !self.new_number && self.display.len() > 1 {
                                            self.display.pop();
                                        } else {
                                            self.display = "0".to_string();
                                            self.new_number = true;
                                        }
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("±")).clicked()
                                    {
                                        let val = self.get_display_value();
                                        self.display = format_number(-val);
                                    }
                                });

                                // Number pad and operations
                                ui.horizontal(|ui| {
                                    if ui.add_sized(button_size, egui::Button::new("7")).clicked() {
                                        self.append_digit("7");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("8")).clicked() {
                                        self.append_digit("8");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("9")).clicked() {
                                        self.append_digit("9");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("÷")).clicked()
                                    {
                                        self.set_operation(Operation::Divide);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.add_sized(button_size, egui::Button::new("4")).clicked() {
                                        self.append_digit("4");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("5")).clicked() {
                                        self.append_digit("5");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("6")).clicked() {
                                        self.append_digit("6");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("×")).clicked()
                                    {
                                        self.set_operation(Operation::Multiply);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.add_sized(button_size, egui::Button::new("1")).clicked() {
                                        self.append_digit("1");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("2")).clicked() {
                                        self.append_digit("2");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("3")).clicked() {
                                        self.append_digit("3");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("−")).clicked()
                                    {
                                        self.set_operation(Operation::Subtract);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.add_sized(button_size, egui::Button::new("0")).clicked() {
                                        self.append_digit("0");
                                    }
                                    if ui.add_sized(button_size, egui::Button::new(".")).clicked() {
                                        self.append_digit(".");
                                    }
                                    if ui
                                        .add_sized(
                                            button_size,
                                            egui::Button::new("=")
                                                .fill(Color32::from_rgb(0, 200, 0)),
                                        )
                                        .clicked()
                                    {
                                        self.calculate();
                                    }
                                    if ui.add_sized(button_size, egui::Button::new("+")).clicked() {
                                        self.set_operation(Operation::Add);
                                    }
                                });

                                // Advanced operations
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(button_size, egui::Button::new("x^y"))
                                        .clicked()
                                    {
                                        self.set_operation(Operation::Power);
                                    }
                                    if ui
                                        .add_sized(button_size, egui::Button::new("y-Root"))
                                        .clicked()
                                    {
                                        self.set_operation(Operation::Root);
                                    }
                                    if ui
                                        .add_sized(button_size, egui::Button::new("mod"))
                                        .clicked()
                                    {
                                        self.set_operation(Operation::Modulo);
                                    }
                                    if ui
                                        .add_sized(button_size, egui::Button::new("1/x"))
                                        .clicked()
                                    {
                                        self.apply_function(|x| {
                                            if x != 0.0 {
                                                1.0 / x
                                            } else {
                                                f64::INFINITY
                                            }
                                        });
                                    }
                                });

                                ui.add_space(15.0);

                                // Expression input field
                                ui.horizontal(|ui| {
                                    ui.label("Expression:");
                                    let response =
                                        ui.text_edit_singleline(&mut self.expression_input);

                                    if response.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    {
                                        self.evaluate_expression();
                                    }

                                    if ui.button("Evaluate").clicked() {
                                        self.evaluate_expression();
                                    }
                                });
                            }); // Close left column vertical

                            ui.add_space(15.0);

                            // Right column: Base conversion and bitwise operations
                            ui.vertical(|ui| {
                                ui.label(format!("Mode: {}", self.base_mode));
                                ui.add_space(5.0);
                                ui.label("Base Conversion:");
                                ui.horizontal(|ui| {
                                    if ui.button("DEC").clicked() {
                                        self.convert_base("DEC");
                                    }
                                    if ui.button("BIN").clicked() {
                                        self.convert_base("BIN");
                                    }
                                    if ui.button("OCT").clicked() {
                                        self.convert_base("OCT");
                                    }
                                    if ui.button("HEX").clicked() {
                                        self.convert_base("HEX");
                                    }
                                });

                                ui.add_space(10.0);
                                ui.label("Bitwise Operations:");

                                // NOT operation (unary)
                                ui.horizontal(|ui| {
                                    if ui.button("NOT").clicked() {
                                        self.apply_bitwise_not();
                                    }
                                });

                                ui.add_space(5.0);

                                // Binary operations
                                ui.horizontal(|ui| {
                                    if ui.button("AND").clicked() {
                                        self.set_bitwise_operation("AND");
                                    }
                                    if ui.button("OR").clicked() {
                                        self.set_bitwise_operation("OR");
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("XOR").clicked() {
                                        self.set_bitwise_operation("XOR");
                                    }
                                    if ui.button("NAND").clicked() {
                                        self.set_bitwise_operation("NAND");
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("NOR").clicked() {
                                        self.set_bitwise_operation("NOR");
                                    }
                                    if ui.button("XNOR").clicked() {
                                        self.set_bitwise_operation("XNOR");
                                    }
                                });

                                ui.add_space(5.0);
                                ui.label("Bit Shifts:");
                                ui.horizontal(|ui| {
                                    if ui.button("<<").clicked() {
                                        self.apply_shift_left();
                                    }
                                    if ui.button(">>").clicked() {
                                        self.apply_shift_right();
                                    }
                                });

                                ui.add_space(10.0);
                                ui.label("Programmer Tools:");

                                ui.horizontal(|ui| {
                                    if ui.button("ASCII").clicked() {
                                        self.show_ascii_value();
                                    }
                                    if ui.button("2's Comp").clicked() {
                                        self.apply_twos_complement();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("BitCount").clicked() {
                                        self.count_bits();
                                    }
                                    if ui.button("ROR").clicked() {
                                        self.apply_rotate_right();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("ROL").clicked() {
                                        self.apply_rotate_left();
                                    }
                                    if ui.button("Abs").clicked() {
                                        self.apply_function(|x| x.abs());
                                    }
                                });

                                ui.add_space(15.0);
                                ui.separator();
                                ui.add_space(5.0);
                                ui.label("Statistics:");

                                ui.horizontal(|ui| {
                                    if ui.button("Add Data").clicked() {
                                        self.stat_add_data();
                                    }
                                    if ui.button("Clear Data").clicked() {
                                        self.stat_clear();
                                    }
                                });

                                // Data display window - Resizable
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .min_scrolled_width(250.0)
                                        .min_scrolled_height(200.0)
                                        .max_height(400.0)
                                        .show(ui, |ui| {
                                            ui.set_min_width(250.0);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "Data ({} items):",
                                                    self.stat_data.len()
                                                ))
                                                .strong(),
                                            );
                                            ui.separator();
                                            if self.stat_data.is_empty() {
                                                ui.label("(no data)");
                                            } else {
                                                for (i, value) in self.stat_data.iter().enumerate()
                                                {
                                                    ui.label(format!(
                                                        "{}. {}",
                                                        i + 1,
                                                        format_number(*value)
                                                    ));
                                                }
                                            }
                                        });
                                });

                                ui.add_space(5.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Mean").clicked() {
                                        self.stat_mean();
                                    }
                                    if ui.button("Sum").clicked() {
                                        self.stat_sum();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("Count").clicked() {
                                        self.stat_count();
                                    }
                                    if ui.button("Std Dev").clicked() {
                                        self.stat_std_dev();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("Variance").clicked() {
                                        self.stat_variance();
                                    }
                                });

                                ui.add_space(10.0);
                                ui.label("Probability:");

                                ui.horizontal(|ui| {
                                    if ui.button("nPr").clicked() {
                                        self.set_operation(Operation::Permutation);
                                    }
                                    if ui.button("nCr").clicked() {
                                        self.set_operation(Operation::Combination);
                                    }
                                });
                            });
                        }); // Close horizontal for main content
                    });
                });
            });
        });
    }
}
