// SPDX-License-Identifier: MIT OR Apache-2.0

//! PostScript Type 4 (calculator) function evaluator.
//!
//! PDF Type 4 functions are small stack-based programs used as tint transforms
//! in Separation and DeviceN color spaces. This module parses and evaluates
//! them per PDF spec Table 42.
//!
//! # Integration
//!
//! The renderer at `src/rendering/page_renderer.rs` (lines 566-642) currently
//! handles Type 2 (exponential interpolation) tint transforms and falls back
//! to grayscale for everything else. To support Type 4, add a branch for
//! `FunctionType == 4`: decode the function stream, then call
//! `evaluate_type4(stream_bytes, &[tint])` to get CMYK components.

#![forbid(unsafe_code)]

use crate::error::{Error, Result};

/// A parsed instruction in a Type 4 PostScript calculator program.
#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    Operand(f64),
    BoolLiteral(bool),
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Neg,
    Abs,
    Ceiling,
    Floor,
    Round,
    Truncate,
    Sqrt,
    Exp,
    Ln,
    Log,
    Sin,
    Cos,
    Atan,
    // Comparison
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    // Boolean/bitwise
    And,
    Or,
    Xor,
    Not,
    Bitshift,
    // Stack manipulation
    Dup,
    Exch,
    Pop,
    Copy,
    Index,
    Roll,
    // Conditional
    If(Vec<Instruction>),
    IfElse(Vec<Instruction>, Vec<Instruction>),
}

/// Parse a Type 4 PostScript calculator program from raw bytes.
///
/// The program must be enclosed in `{ }`. Nested braces define procedure
/// bodies used with `if` and `ifelse`.
fn parse(program: &[u8]) -> Result<Vec<Instruction>> {
    let s = std::str::from_utf8(program)
        .map_err(|e| Error::InvalidPdf(format!("Type 4 function is not valid UTF-8: {e}")))?;
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return Err(Error::InvalidPdf("Type 4 function must be enclosed in { }".into()));
    }
    let inner = &s[1..s.len() - 1];
    parse_body(inner)
}

fn parse_body(s: &str) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some(&(i, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '{' {
            chars.next();
            let start = if let Some(&(idx, _)) = chars.peek() {
                idx
            } else {
                return Err(Error::InvalidPdf("Unclosed brace in Type 4 function".into()));
            };
            let mut depth = 1u32;
            let mut end = start;
            for (j, ch) in chars.by_ref() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end = j;
                        break;
                    }
                }
            }
            if depth != 0 {
                return Err(Error::InvalidPdf("Unclosed brace in Type 4 function".into()));
            }
            let body = parse_body(&s[start..end])?;
            instructions.push(Instruction::If(body));
            continue;
        }
        // Collect a token
        let start = i;
        while let Some(&(_, tc)) = chars.peek() {
            if tc.is_whitespace() || tc == '{' || tc == '}' {
                break;
            }
            chars.next();
        }
        let end = if let Some(&(idx, _)) = chars.peek() {
            idx
        } else {
            s.len()
        };
        let token = &s[start..end];
        instructions.push(parse_token(token)?);
    }

    // Post-process: resolve `if` and `ifelse` by consuming preceding procedure bodies.
    resolve_conditionals(&mut instructions)?;
    Ok(instructions)
}

fn parse_token(token: &str) -> Result<Instruction> {
    match token {
        "add" => Ok(Instruction::Add),
        "sub" => Ok(Instruction::Sub),
        "mul" => Ok(Instruction::Mul),
        "div" => Ok(Instruction::Div),
        "idiv" => Ok(Instruction::Idiv),
        "mod" => Ok(Instruction::Mod),
        "neg" => Ok(Instruction::Neg),
        "abs" => Ok(Instruction::Abs),
        "ceiling" => Ok(Instruction::Ceiling),
        "floor" => Ok(Instruction::Floor),
        "round" => Ok(Instruction::Round),
        "truncate" => Ok(Instruction::Truncate),
        "sqrt" => Ok(Instruction::Sqrt),
        "exp" => Ok(Instruction::Exp),
        "ln" => Ok(Instruction::Ln),
        "log" => Ok(Instruction::Log),
        "sin" => Ok(Instruction::Sin),
        "cos" => Ok(Instruction::Cos),
        "atan" => Ok(Instruction::Atan),
        "eq" => Ok(Instruction::Eq),
        "ne" => Ok(Instruction::Ne),
        "gt" => Ok(Instruction::Gt),
        "ge" => Ok(Instruction::Ge),
        "lt" => Ok(Instruction::Lt),
        "le" => Ok(Instruction::Le),
        "and" => Ok(Instruction::And),
        "or" => Ok(Instruction::Or),
        "xor" => Ok(Instruction::Xor),
        "not" => Ok(Instruction::Not),
        "bitshift" => Ok(Instruction::Bitshift),
        "true" => Ok(Instruction::BoolLiteral(true)),
        "false" => Ok(Instruction::BoolLiteral(false)),
        "dup" => Ok(Instruction::Dup),
        "exch" => Ok(Instruction::Exch),
        "pop" => Ok(Instruction::Pop),
        "copy" => Ok(Instruction::Copy),
        "index" => Ok(Instruction::Index),
        "roll" => Ok(Instruction::Roll),
        "if" | "ifelse" => Ok(if token == "if" {
            // Placeholder; resolved in post-processing
            Instruction::If(vec![])
        } else {
            Instruction::IfElse(vec![], vec![])
        }),
        _ => {
            let val: f64 = token
                .parse()
                .map_err(|_| Error::InvalidPdf(format!("Unknown Type 4 token: {token}")))?;
            Ok(Instruction::Operand(val))
        },
    }
}

/// Post-process: attach preceding procedure bodies to `if`/`ifelse` instructions.
fn resolve_conditionals(instructions: &mut Vec<Instruction>) -> Result<()> {
    let mut i = 0;
    while i < instructions.len() {
        match &instructions[i] {
            Instruction::If(body) if body.is_empty() => {
                // `if`: one preceding procedure body
                if i == 0 {
                    return Err(Error::InvalidPdf(
                        "Type 4 `if` without preceding procedure body".into(),
                    ));
                }
                if let Instruction::If(body) = instructions.remove(i - 1) {
                    instructions[i - 1] = Instruction::If(body);
                    // Don't increment i; we removed an element before
                } else {
                    return Err(Error::InvalidPdf("Type 4 `if` requires a procedure body".into()));
                }
            },
            Instruction::IfElse(_, _) => {
                // `ifelse`: two preceding procedure bodies
                if i < 2 {
                    return Err(Error::InvalidPdf(
                        "Type 4 `ifelse` without two preceding procedure bodies".into(),
                    ));
                }
                let false_branch = match instructions.remove(i - 1) {
                    Instruction::If(body) => body,
                    _ => {
                        return Err(Error::InvalidPdf(
                            "Type 4 `ifelse` requires two procedure bodies".into(),
                        ))
                    },
                };
                let true_branch = match instructions.remove(i - 2) {
                    Instruction::If(body) => body,
                    _ => {
                        return Err(Error::InvalidPdf(
                            "Type 4 `ifelse` requires two procedure bodies".into(),
                        ))
                    },
                };
                instructions[i - 2] = Instruction::IfElse(true_branch, false_branch);
                i = i.saturating_sub(1);
            },
            _ => {
                i += 1;
            },
        }
    }
    Ok(())
}

/// Evaluate a Type 4 PostScript calculator program.
///
/// `program` is the raw stream content (e.g. `{ dup 0.84 mul ... }`).
/// `inputs` are pushed onto the stack before execution.
/// After execution the remaining stack values are returned as the output.
pub fn evaluate_type4(program: &[u8], inputs: &[f64]) -> Result<Vec<f64>> {
    let instructions = parse(program)?;
    let mut stack: Vec<f64> = inputs.to_vec();
    execute(&instructions, &mut stack)?;
    Ok(stack)
}

/// Evaluate with Domain/Range clamping per the PDF function dictionary.
///
/// `domain` is a list of `[min, max]` pairs (one per input). Each input is
/// clamped to its domain before execution. `range` is a list of `[min, max]`
/// pairs (one per output). Each output is clamped to its range after execution.
pub fn evaluate_type4_clamped(
    program: &[u8],
    inputs: &[f64],
    domain: &[[f64; 2]],
    range: &[[f64; 2]],
) -> Result<Vec<f64>> {
    let clamped_inputs: Vec<f64> = inputs
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            if let Some(&[lo, hi]) = domain.get(i) {
                v.clamp(lo, hi)
            } else {
                v
            }
        })
        .collect();
    let mut result = evaluate_type4(program, &clamped_inputs)?;
    for (i, val) in result.iter_mut().enumerate() {
        if let Some(&[lo, hi]) = range.get(i) {
            *val = val.clamp(lo, hi);
        }
    }
    Ok(result)
}

fn execute(instructions: &[Instruction], stack: &mut Vec<f64>) -> Result<()> {
    for instr in instructions {
        match instr {
            Instruction::Operand(v) => stack.push(*v),
            Instruction::BoolLiteral(b) => stack.push(if *b { 1.0 } else { 0.0 }),
            Instruction::Add => binary_op(stack, |a, b| Ok(a + b))?,
            Instruction::Sub => binary_op(stack, |a, b| Ok(a - b))?,
            Instruction::Mul => binary_op(stack, |a, b| Ok(a * b))?,
            Instruction::Div => binary_op(stack, |a, b| {
                if b == 0.0 {
                    Err(Error::InvalidPdf("Type 4 division by zero".into()))
                } else {
                    Ok(a / b)
                }
            })?,
            Instruction::Idiv => binary_op(stack, |a, b| {
                let ib = b as i64;
                if ib == 0 {
                    Err(Error::InvalidPdf("Type 4 idiv by zero".into()))
                } else {
                    Ok(((a as i64) / ib) as f64)
                }
            })?,
            Instruction::Mod => binary_op(stack, |a, b| {
                let ib = b as i64;
                if ib == 0 {
                    Err(Error::InvalidPdf("Type 4 mod by zero".into()))
                } else {
                    Ok(((a as i64) % ib) as f64)
                }
            })?,
            Instruction::Neg => unary_op(stack, |a| Ok(-a))?,
            Instruction::Abs => unary_op(stack, |a| Ok(a.abs()))?,
            Instruction::Ceiling => unary_op(stack, |a| Ok(a.ceil()))?,
            Instruction::Floor => unary_op(stack, |a| Ok(a.floor()))?,
            Instruction::Round => unary_op(stack, |a| Ok(a.round()))?,
            Instruction::Truncate => unary_op(stack, |a| Ok(a.trunc()))?,
            Instruction::Sqrt => unary_op(stack, |a| Ok(a.sqrt()))?,
            Instruction::Exp => binary_op(stack, |base, exp| Ok(base.powf(exp)))?,
            Instruction::Ln => unary_op(stack, |a| Ok(a.ln()))?,
            Instruction::Log => unary_op(stack, |a| Ok(a.log10()))?,
            Instruction::Sin => unary_op(stack, |a| Ok(a.to_radians().sin()))?,
            Instruction::Cos => unary_op(stack, |a| Ok(a.to_radians().cos()))?,
            Instruction::Atan => binary_op(stack, |num, den| Ok(num.atan2(den).to_degrees()))?,
            Instruction::Eq => binary_op(stack, |a, b| Ok(bool_val(a == b)))?,
            Instruction::Ne => binary_op(stack, |a, b| Ok(bool_val(a != b)))?,
            Instruction::Gt => binary_op(stack, |a, b| Ok(bool_val(a > b)))?,
            Instruction::Ge => binary_op(stack, |a, b| Ok(bool_val(a >= b)))?,
            Instruction::Lt => binary_op(stack, |a, b| Ok(bool_val(a < b)))?,
            Instruction::Le => binary_op(stack, |a, b| Ok(bool_val(a <= b)))?,
            Instruction::And => binary_op(stack, |a, b| {
                Ok(if is_bool(a) && is_bool(b) {
                    bool_val(as_bool(a) && as_bool(b))
                } else {
                    ((a as i64) & (b as i64)) as f64
                })
            })?,
            Instruction::Or => binary_op(stack, |a, b| {
                Ok(if is_bool(a) && is_bool(b) {
                    bool_val(as_bool(a) || as_bool(b))
                } else {
                    ((a as i64) | (b as i64)) as f64
                })
            })?,
            Instruction::Xor => binary_op(stack, |a, b| {
                Ok(if is_bool(a) && is_bool(b) {
                    bool_val(as_bool(a) ^ as_bool(b))
                } else {
                    ((a as i64) ^ (b as i64)) as f64
                })
            })?,
            Instruction::Not => unary_op(stack, |a| {
                Ok(if is_bool(a) {
                    bool_val(!as_bool(a))
                } else {
                    (!(a as i64)) as f64
                })
            })?,
            Instruction::Bitshift => binary_op(stack, |val, shift| {
                let iv = val as i64;
                let is = shift as i64;
                Ok(if is >= 0 { iv << is } else { iv >> (-is) } as f64)
            })?,
            Instruction::Dup => {
                let a = pop(stack)?;
                stack.push(a);
                stack.push(a);
            },
            Instruction::Exch => {
                let b = pop(stack)?;
                let a = pop(stack)?;
                stack.push(b);
                stack.push(a);
            },
            Instruction::Pop => {
                pop(stack)?;
            },
            Instruction::Copy => {
                let n = pop(stack)? as usize;
                if n > stack.len() {
                    return Err(underflow());
                }
                let start = stack.len() - n;
                let copied: Vec<f64> = stack[start..].to_vec();
                stack.extend_from_slice(&copied);
            },
            Instruction::Index => {
                let n = pop(stack)? as usize;
                if n >= stack.len() {
                    return Err(underflow());
                }
                let val = stack[stack.len() - 1 - n];
                stack.push(val);
            },
            Instruction::Roll => {
                let j = pop(stack)? as i64;
                let n = pop(stack)? as usize;
                if n > stack.len() {
                    return Err(underflow());
                }
                if n > 0 {
                    let start = stack.len() - n;
                    let slice = &mut stack[start..];
                    let len = slice.len();
                    let shift = ((j % len as i64) + len as i64) as usize % len;
                    slice.rotate_right(shift);
                }
            },
            Instruction::If(body) => {
                let cond = pop(stack)?;
                if as_bool(cond) {
                    execute(body, stack)?;
                }
            },
            Instruction::IfElse(true_branch, false_branch) => {
                let cond = pop(stack)?;
                if as_bool(cond) {
                    execute(true_branch, stack)?;
                } else {
                    execute(false_branch, stack)?;
                }
            },
        }
    }
    Ok(())
}

fn pop(stack: &mut Vec<f64>) -> Result<f64> {
    stack.pop().ok_or_else(underflow)
}

fn underflow() -> Error {
    Error::InvalidPdf("Type 4 stack underflow".into())
}

fn unary_op(stack: &mut Vec<f64>, f: impl FnOnce(f64) -> Result<f64>) -> Result<()> {
    let a = pop(stack)?;
    stack.push(f(a)?);
    Ok(())
}

fn binary_op(stack: &mut Vec<f64>, f: impl FnOnce(f64, f64) -> Result<f64>) -> Result<()> {
    let b = pop(stack)?;
    let a = pop(stack)?;
    stack.push(f(a, b)?);
    Ok(())
}

fn bool_val(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

fn as_bool(v: f64) -> bool {
    v != 0.0
}

fn is_bool(v: f64) -> bool {
    v == 0.0 || v == 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &[f64], b: &[f64], eps: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < eps)
    }

    #[test]
    fn linear_ramp_tint_transform() {
        let prog = b"{ dup 0.84 mul exch 0.00 exch dup 0.44 mul exch 0.21 mul }";
        let result = evaluate_type4(prog, &[0.5]).unwrap();
        assert!(approx_eq(&result, &[0.42, 0.0, 0.22, 0.105], 1e-9), "got {result:?}");
    }

    #[test]
    fn identity_empty_program() {
        let result = evaluate_type4(b"{ }", &[0.7]).unwrap();
        assert!(approx_eq(&result, &[0.7], 1e-9), "got {result:?}");
    }

    #[test]
    fn constant_output() {
        let prog = b"{ pop 1.0 0.0 0.0 0.0 }";
        let result = evaluate_type4(prog, &[0.5]).unwrap();
        assert_eq!(result, vec![1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn conditional_ifelse() {
        let prog = b"{ dup 0.5 gt { pop 1.0 } { 0.0 exch } ifelse }";
        let high = evaluate_type4(prog, &[0.8]).unwrap();
        assert_eq!(high, vec![1.0]);
        let low = evaluate_type4(prog, &[0.3]).unwrap();
        assert!(approx_eq(&low, &[0.0, 0.3], 1e-9), "got {low:?}");
    }

    #[test]
    fn conditional_if() {
        let prog = b"{ dup 0.5 gt { 1.0 add } if }";
        let high = evaluate_type4(prog, &[0.8]).unwrap();
        assert!(approx_eq(&high, &[1.8], 1e-9), "got {high:?}");
        let low = evaluate_type4(prog, &[0.3]).unwrap();
        assert!(approx_eq(&low, &[0.3], 1e-9), "got {low:?}");
    }

    #[test]
    fn domain_range_clamping() {
        let prog = b"{ 2.0 mul }";
        let result = evaluate_type4_clamped(prog, &[1.5], &[[0.0, 1.0]], &[[0.0, 1.0]]).unwrap();
        // Input 1.5 clamped to 1.0, * 2.0 = 2.0, clamped to 1.0
        assert_eq!(result, vec![1.0]);
    }

    #[test]
    fn stack_underflow_returns_error() {
        let prog = b"{ add }";
        let err = evaluate_type4(prog, &[]).unwrap_err();
        assert!(err.to_string().contains("stack underflow"), "got: {err}");
    }

    #[test]
    fn arithmetic_operators() {
        assert_eq!(evaluate_type4(b"{ add }", &[3.0, 4.0]).unwrap(), vec![7.0]);
        assert_eq!(evaluate_type4(b"{ sub }", &[10.0, 3.0]).unwrap(), vec![7.0]);
        assert_eq!(evaluate_type4(b"{ mul }", &[3.0, 4.0]).unwrap(), vec![12.0]);
        assert_eq!(evaluate_type4(b"{ div }", &[10.0, 4.0]).unwrap(), vec![2.5]);
        assert_eq!(evaluate_type4(b"{ idiv }", &[10.0, 3.0]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ mod }", &[10.0, 3.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ neg }", &[5.0]).unwrap(), vec![-5.0]);
        assert_eq!(evaluate_type4(b"{ abs }", &[-5.0]).unwrap(), vec![5.0]);
        assert_eq!(evaluate_type4(b"{ ceiling }", &[3.2]).unwrap(), vec![4.0]);
        assert_eq!(evaluate_type4(b"{ floor }", &[3.8]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ round }", &[3.5]).unwrap(), vec![4.0]);
        assert_eq!(evaluate_type4(b"{ truncate }", &[3.9]).unwrap(), vec![3.0]);
        assert_eq!(evaluate_type4(b"{ sqrt }", &[9.0]).unwrap(), vec![3.0]);
    }

    #[test]
    fn trig_operators() {
        let sin_result = evaluate_type4(b"{ sin }", &[90.0]).unwrap();
        assert!((sin_result[0] - 1.0).abs() < 1e-9);
        let cos_result = evaluate_type4(b"{ cos }", &[0.0]).unwrap();
        assert!((cos_result[0] - 1.0).abs() < 1e-9);
        let atan_result = evaluate_type4(b"{ atan }", &[1.0, 1.0]).unwrap();
        assert!((atan_result[0] - 45.0).abs() < 1e-9);
    }

    #[test]
    fn log_operators() {
        let ln_result = evaluate_type4(b"{ ln }", &[std::f64::consts::E]).unwrap();
        assert!((ln_result[0] - 1.0).abs() < 1e-9);
        let log_result = evaluate_type4(b"{ log }", &[100.0]).unwrap();
        assert!((log_result[0] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn exp_operator() {
        let result = evaluate_type4(b"{ exp }", &[2.0, 10.0]).unwrap();
        assert!((result[0] - 1024.0).abs() < 1e-9);
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(evaluate_type4(b"{ eq }", &[1.0, 1.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ eq }", &[1.0, 2.0]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ ne }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ gt }", &[2.0, 1.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ ge }", &[2.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ lt }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ le }", &[2.0, 2.0]).unwrap(), vec![1.0]);
    }

    #[test]
    fn boolean_operators() {
        assert_eq!(evaluate_type4(b"{ and }", &[1.0, 0.0]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ or }", &[1.0, 0.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ xor }", &[1.0, 1.0]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ not }", &[1.0]).unwrap(), vec![0.0]);
        assert_eq!(evaluate_type4(b"{ not }", &[0.0]).unwrap(), vec![1.0]);
    }

    #[test]
    fn bitwise_operators() {
        assert_eq!(evaluate_type4(b"{ and }", &[12.0, 10.0]).unwrap(), vec![8.0]);
        assert_eq!(evaluate_type4(b"{ or }", &[12.0, 10.0]).unwrap(), vec![14.0]);
        assert_eq!(evaluate_type4(b"{ bitshift }", &[8.0, 2.0]).unwrap(), vec![32.0]);
        assert_eq!(evaluate_type4(b"{ bitshift }", &[32.0, -2.0]).unwrap(), vec![8.0]);
    }

    #[test]
    fn stack_manipulation() {
        assert_eq!(evaluate_type4(b"{ dup }", &[5.0]).unwrap(), vec![5.0, 5.0]);
        assert_eq!(evaluate_type4(b"{ exch }", &[1.0, 2.0]).unwrap(), vec![2.0, 1.0]);
        assert_eq!(evaluate_type4(b"{ pop }", &[1.0, 2.0]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ 2 copy }", &[1.0, 2.0]).unwrap(), vec![1.0, 2.0, 1.0, 2.0]);
        assert_eq!(evaluate_type4(b"{ 1 index }", &[1.0, 2.0]).unwrap(), vec![1.0, 2.0, 1.0]);
    }

    #[test]
    fn roll_operator() {
        // roll(n=3, j=1): rotate top 3 elements by 1
        // [1, 2, 3] -> [3, 1, 2]
        assert_eq!(evaluate_type4(b"{ 3 1 roll }", &[1.0, 2.0, 3.0]).unwrap(), vec![3.0, 1.0, 2.0]);
        // roll(n=3, j=-1): rotate top 3 elements by -1
        // [1, 2, 3] -> [2, 3, 1]
        assert_eq!(
            evaluate_type4(b"{ 3 -1 roll }", &[1.0, 2.0, 3.0]).unwrap(),
            vec![2.0, 3.0, 1.0]
        );
    }

    #[test]
    fn bool_literals() {
        assert_eq!(evaluate_type4(b"{ true }", &[]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(b"{ false }", &[]).unwrap(), vec![0.0]);
    }

    #[test]
    fn division_by_zero() {
        let err = evaluate_type4(b"{ div }", &[1.0, 0.0]).unwrap_err();
        assert!(err.to_string().contains("division by zero"), "got: {err}");
    }

    #[test]
    fn invalid_program_missing_braces() {
        let err = evaluate_type4(b"dup mul", &[1.0]).unwrap_err();
        assert!(err.to_string().contains("{ }"), "got: {err}");
    }

    #[test]
    fn nested_conditionals() {
        let prog =
            b"{ dup 0.5 gt { dup 0.8 gt { pop 1.0 } { pop 0.75 } ifelse } { pop 0.0 } ifelse }";
        assert_eq!(evaluate_type4(prog, &[0.9]).unwrap(), vec![1.0]);
        assert_eq!(evaluate_type4(prog, &[0.6]).unwrap(), vec![0.75]);
        assert_eq!(evaluate_type4(prog, &[0.3]).unwrap(), vec![0.0]);
    }

    #[test]
    fn real_world_spot_color_transforms() {
        // Pantone-style: single ink maps to CMYK
        let prog = b"{ 0 exch dup 0.78 mul exch 0.35 mul 0 }";
        let result = evaluate_type4(prog, &[1.0]).unwrap();
        assert!(approx_eq(&result, &[0.0, 0.78, 0.35, 0.0], 1e-9), "got {result:?}");
    }

    #[test]
    fn negative_number_literal() {
        let result = evaluate_type4(b"{ -3.5 add }", &[10.0]).unwrap();
        assert!(approx_eq(&result, &[6.5], 1e-9), "got {result:?}");
    }
}
