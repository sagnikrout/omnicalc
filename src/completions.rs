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
                "\n# omnicalc physical units and built-in math functions\n_omnicalc_math_words=\"{}\"\n_omnicalc_with_units() {{\n    local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n    _omnicalc \"$@\"\n    if [[ \"${{cur}}\" != -* && \"${{COMP_WORDS[COMP_CWORD-1]:-}}\" != \"--completions\" && \"${{COMP_WORDS[COMP_CWORD-1]:-}}\" != \"-b\" && \"${{COMP_WORDS[COMP_CWORD-1]:-}}\" != \"--base\" ]]; then\n        COMPREPLY+=( $(compgen -W \"${{_omnicalc_math_words}}\" -- \"${{cur}}\") )\n    fi\n}}\ncomplete -F _omnicalc_with_units -o nosort -o bashdefault -o default omnicalc\n",
                joined
            );
            buf.extend_from_slice(extra.as_bytes());
        }
        Shell::Zsh => {
            let mut s = String::from_utf8_lossy(&buf).into_owned();
            s = s.replace(
                "*::expression -- Positional expression arguments:_default",
                "*::expression -- Positional expression arguments:_omnicalc_expr",
            );
            let joined = words.join(" ");
            s.push_str(&format!(
                "\n_omnicalc_expr() {{\n    local -a _omnicalc_math_words\n    _omnicalc_math_words=({})\n    _describe -t units-and-functions 'unit or function' _omnicalc_math_words\n}}\n",
                joined
            ));
            buf = s.into_bytes();
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
