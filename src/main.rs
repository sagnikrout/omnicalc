use clap::Parser;
use omnicalc::eval::Evaluator;
use omnicalc::lexer::Lexer;
use omnicalc::parser::Parser as CalcParser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::io::{self, BufRead, IsTerminal};

#[derive(Parser, Debug)]
#[command(name = "omnicalc")]
#[command(author = "Sagnik Rout & Gemini 3.8 Flash High (Google DeepMind)")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "Unified high-performance scientific CLI calculator & CAS (https://github.com/sagnikrout/omnicalc)"
)]
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

    /// Display credits, project vision, and acknowledgments
    #[arg(long = "credits")]
    credits: bool,

    /// Display project vision and architectural origins
    #[arg(long = "about")]
    about: bool,
}

fn evaluate_str(
    evaluator: &mut Evaluator,
    input: &str,
    raw: bool,
    base_code: u32,
) -> Result<String, String> {
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

fn print_or_exit(eval_result: Result<String, String>) {
    match eval_result {
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

    if args.credits || args.about {
        print_credits();
        return;
    }

    let base_code = parse_base_str(&args.base);
    let mut evaluator = Evaluator::new();

    // Check for expression from -e or positional args
    let mut direct_expr = args.eval;
    if direct_expr.is_none() && !args.expression.is_empty() {
        direct_expr = Some(args.expression.join(" "));
    }

    if let Some(expr_str) = direct_expr {
        print_or_exit(evaluate_str(&mut evaluator, &expr_str, args.raw, base_code));
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
                    print_or_exit(evaluate_str(&mut evaluator, trimmed, args.raw, base_code));
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
    println!(
        "omnicalc v{} — Unified CLI Scientific & Physical Engine",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "Origin: Sagnik Rout (https://github.com/sagnikrout/omnicalc) | Synthesis: Gemini 3.8 Flash High"
    );
    println!(
        "Synthesis: qalc (CAS/Uncertainty) + Numbat (Dimensional Safety) + fend (Ergonomics) + Kalker (Math Syntax)"
    );
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

fn print_credits() {
    println!("omnicalc — Version {}", env!("CARGO_PKG_VERSION"));
    println!("Repository: https://github.com/sagnikrout/omnicalc");
    println!("High-Performance Scientific CLI Calculator & Computer Algebra System\n");
    println!("=== Origin & Vision ===");
    println!("  Conceived and founded by Sagnik Rout (https://github.com/sagnikrout)");
    println!("  Started years ago with a bold vision and relentless optimism: to build a unified,");
    println!("  lightning-fast, zero-compromise scientific calculator for the UNIX terminal.\n");
    println!("=== Engineering & Synthesis ===");
    println!("  Co-engineered with Gemini 3.8 Flash High & Antigravity (Google DeepMind)");
    println!("  Operating in Charlie Rutledge mode: rigorous verification, boundary defense,");
    println!("  and zero-handwaving engineering standards.\n");
    println!("=== Standing on the Shoulders of Giants ===");
    println!("  We express deep gratitude to the creators and maintainers of the open-source");
    println!(
        "  scientific tools whose years of work established the foundation for modern CLI math:"
    );
    println!(
        "  - Qalculate! / libqalculate (Niklas Knutsson & contributors) — CAS, dimensional units, uncertainty."
    );
    println!(
        "  - Numbat (David Peter / @sharkdp & contributors) — Rigorous static physical dimension typing."
    );
    println!(
        "  - fend (Evan Jones / @printfn & contributors) — Exact rational AST, elegant CLI ergonomics."
    );
    println!(
        "  - kalker (PaddiM8 & contributors) — Pure mathematical syntax, complex roots, matrices."
    );
    println!("  - Insect (David Peter) — The web/CLI physical unit predecessor.");
    println!(
        "  - calc / apcalc (David I. Bell, Landon Curt Noll) — C-based arbitrary precision pioneer since the 1990s.\n"
    );
    println!("=== Mathematical Foundations ===");
    println!("  Dedicated to the mathematicians whose theorems power this engine:");
    println!("  Leonhard Euler, Isaac Newton, Joseph Raphson, Carl Friedrich Gauss,");
    println!("  Alexander Kronrod, Thomas Simpson, Vaughan Pratt, and Bernhard Riemann.");
}
