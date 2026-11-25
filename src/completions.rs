use clap::Command;
use clap_complete::{Shell, generate};

pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "abs",
    "acos",
    "asin",
    "atan",
    "cbrt",
    "ceil",
    "cos",
    "cross",
    "det",
    "diff",
    "dot",
    "exp",
    "fact",
    "factorial",
    "factor",
    "floor",
    "gcd",
    "int",
    "integrate",
    "inv",
    "lcm",
    "ln",
    "log",
    "log10",
    "log2",
    "norm",
    "normdist",
    "round",
    "sin",
    "solve",
    "sqrt",
    "tan",
    "transpose",
];

pub fn generate_completions(shell: Shell, cmd: &mut Command, units: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();
    generate(shell, cmd, "omnicalc", &mut buf);

    let mut words: Vec<String> = units.to_vec();
    words.extend(BUILTIN_FUNCTIONS.iter().map(|s| (*s).to_string()));
    words.sort();
    words.dedup();

    match shell {
        Shell::Bash => {
            let joined = words.join(" ");
            let extra = format!(
                "\n# omnicalc physical units and built-in math functions\n_omnicalc_math_words=\"{}\"\n",
                joined
            );
            buf.extend_from_slice(extra.as_bytes());
        }
        Shell::Zsh => {
            let joined = words.join(" ");
            let extra = format!(
                "\n# omnicalc physical units and built-in math functions\n# typeset -a _omnicalc_math_words=({})\n",
                joined
            );
            buf.extend_from_slice(extra.as_bytes());
        }
        Shell::Fish => {
            for w in &words {
                let line = format!("complete -c omnicalc -f -a '{}'\n", w);
                buf.extend_from_slice(line.as_bytes());
            }
        }
        _ => {}
    }

    buf
}
