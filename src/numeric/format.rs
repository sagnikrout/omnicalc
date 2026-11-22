use num_traits::ToPrimitive;
use std::fmt;

use super::NumericValue;

impl NumericValue {
    pub fn format_with_base(&self, base: u32, raw: bool) -> String {
        match self {
            Self::Exact(r) if r.is_integer() => {
                let int_val = r.to_integer();
                match base {
                    2 => {
                        if raw {
                            format!("{:b}", int_val)
                        } else {
                            format!("0b{:b}", int_val)
                        }
                    }
                    8 => {
                        if raw {
                            format!("{:o}", int_val)
                        } else {
                            format!("0o{:o}", int_val)
                        }
                    }
                    16 => {
                        if raw {
                            format!("{:x}", int_val)
                        } else {
                            format!("0x{:x}", int_val)
                        }
                    }
                    _ => format!("{}", int_val),
                }
            }
            Self::Exact(r) => {
                if raw {
                    format!("{:.6}", r.to_f64().unwrap_or(0.0))
                } else {
                    format!("{}/{}", r.numer(), r.denom())
                }
            }
            Self::Float(f) => {
                if raw {
                    format!("{}", f)
                } else if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{:.0}", f)
                } else {
                    format!("{:.10}", f)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                }
            }
            Self::Uncertainty { nominal, error } => {
                if raw {
                    format!("{}", nominal)
                } else {
                    format!("{:.4}±{:.4}", nominal, error)
                }
            }
            Self::Complex(c) => {
                if raw {
                    format!("({},{})", c.re, c.im)
                } else if c.im.abs() < 1e-10 {
                    if c.re.fract().abs() < 1e-10 && c.re.abs() < 1e15 {
                        format!("{:.0}", c.re.round())
                    } else {
                        format!("{:.10}", c.re)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string()
                    }
                } else if c.re.abs() < 1e-10 {
                    if (c.im - 1.0).abs() < 1e-10 {
                        "i".to_string()
                    } else if (c.im + 1.0).abs() < 1e-10 {
                        "-i".to_string()
                    } else {
                        let im_str = if c.im.fract().abs() < 1e-10 {
                            format!("{:.0}", c.im.round())
                        } else {
                            format!("{:.10}", c.im)
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                                .to_string()
                        };
                        format!("{}i", im_str)
                    }
                } else {
                    let re_str = if c.re.fract().abs() < 1e-10 {
                        format!("{:.0}", c.re.round())
                    } else {
                        format!("{:.10}", c.re)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string()
                    };
                    let im_abs = c.im.abs();
                    let im_str = if (im_abs - 1.0).abs() < 1e-10 {
                        "i".to_string()
                    } else if im_abs.fract().abs() < 1e-10 {
                        format!("{:.0}i", im_abs.round())
                    } else {
                        format!(
                            "{}i",
                            format!("{:.10}", im_abs)
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                        )
                    };
                    if c.im >= 0.0 {
                        format!("{} + {}", re_str, im_str)
                    } else {
                        format!("{} - {}", re_str, im_str)
                    }
                }
            }
        }
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_with_base(10, false))
    }
}
