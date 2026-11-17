use clap::Parser;
use omnicalc::eval::Evaluator;
use omnicalc::lexer::Lexer;
use omnicalc::parser::Parser as CalcParser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io::{self, BufRead, IsTerminal};

#[derive(Parser, Debug)]
#[command(name = "omnicalc")]
#[command(author = "DeepMind & Antigravity Synthesis Engine")]
#[command(version = "2026.1.0")]
#[command(about = "Unified no-nonsense CLI calculator merging qalc, Numbat, fend, and Kalker in Rust")]
struct Args {
    /// Expression to evaluate directly
    #[arg(short = 'e', long = "eval", allow_hyphen_values = true)]
    eval: Option<String>,

    /// Positional expression arguments
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    expression: Vec<String>,

    /// Raw output mode (no units or decoration, ideal for shell scripts)
    #[arg(short = 'r', long = "raw")]
    raw: bool,

    /// Number base for output (dec, hex, bin, oct)
    #[arg(short = 'b', long = "base", default_value = "dec")]
    base: String,
}

fn evaluate_str(evaluator: &mut Evaluator, input: &str, raw: bool, base_code: u32) -> Result<String, String> {
    let clean = input.trim();
    if clean.is_empty() {
        return Err("Empty input".to_string());
    }

    let mut lexer = Lexer::new(clean);
    let tokens = lexer.tokenize()?;
    let mut parser = CalcParser::new(tokens);
    let ast = parser.parse()?;

    if base_code != 10 {
        let q = evaluator.eval_quantity(&ast)?;
        Ok(q.value.format_with_base(base_code, raw))
    } else if raw {
        evaluator.eval_raw(&ast, 10)
    } else {
        evaluator.eval(&ast)
    }
}

fn parse_base_str(base_str: &str) -> u32 {
    match base_str.to_lowercase().as_str() {
        "hex" | "16" | "x" => 16,
        "bin" | "2" | "b" => 2,
        "oct" | "8" | "o" => 8,
        _ => 10,
    }
}

fn main() {
    let args = Args::parse();
    let base_code = parse_base_str(&args.base);
    let mut evaluator = Evaluator::new();

    // Check for expression from -e or positional args
    let mut direct_expr = args.eval;
    if direct_expr.is_none() && !args.expression.is_empty() {
        direct_expr = Some(args.expression.join(" "));
    }

    if let Some(expr_str) = direct_expr {
        match evaluate_str(&mut evaluator, &expr_str, args.raw, base_code) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
        return;
    }

    // Check if stdin is piped (e.g. echo "5 + 5" | omnicalc)
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        let reader = stdin.lock();
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let trimmed = l.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    match evaluate_str(&mut evaluator, trimmed, args.raw, base_code) {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            std::process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("I/O error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Interactive REPL mode
    println!("omnicalc 2026.1.0 — Unified CLI Scientific & Physical Engine");
    println!("Synthesis: qalc (CAS/Uncertainty) + Numbat (Dimensional Safety) + fend (Ergonomics) + Kalker (Math Syntax)");
    println!("Type 'quit' or 'exit' or press Ctrl-D to exit.\n");

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize interactive line editor: {}", e);
            std::process::exit(1);
        }
    };

    let history_path = dirs_or_home_history();
    let _ = rl.load_history(&history_path);

    loop {
        let readline = rl.readline("omnicalc> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "quit" || trimmed == "exit" {
                    break;
                }
                let _ = rl.add_history_entry(trimmed);

                match evaluate_str(&mut evaluator, trimmed, args.raw, base_code) {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error reading input: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_path);
}

fn dirs_or_home_history() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".omnicalc_history")
    } else {
        std::path::PathBuf::from(".omnicalc_history")
    }
}
