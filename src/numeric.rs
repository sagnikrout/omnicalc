use num_bigint::BigInt;
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    Exact(BigRational),
    Float(f64),
    Uncertainty { nominal: f64, error: f64 },
    Complex(Complex64),
}

impl NumericValue {
    pub fn from_i64(n: i64) -> Self {
        Self::Exact(BigRational::from_integer(BigInt::from(n)))
    }

    pub fn from_f64(f: f64) -> Self {
        Self::Float(f)
    }

    pub fn from_uncertainty(nominal: f64, error: f64) -> Self {
        Self::Uncertainty {
            nominal,
            error: error.abs(),
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Self::Exact(r) => r.to_f64().unwrap_or(f64::NAN),
            Self::Float(f) => *f,
            Self::Uncertainty { nominal, .. } => *nominal,
            Self::Complex(c) => c.re,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Exact(r) => r.is_zero(),
            Self::Float(f) => *f == 0.0,
            Self::Uncertainty { nominal, .. } => *nominal == 0.0,
            Self::Complex(c) => c.is_zero(),
        }
    }

    pub fn is_one(&self) -> bool {
        match self {
            Self::Exact(r) => r.is_one(),
            Self::Float(f) => *f == 1.0,
            Self::Uncertainty { nominal, error } => *nominal == 1.0 && *error == 0.0,
            Self::Complex(c) => c.re == 1.0 && c.im == 0.0,
        }
    }

    pub fn pow(&self, power: &Self) -> Self {
        match (self, power) {
            (Self::Exact(b), Self::Exact(p)) if p.is_integer() => {
                if let Some(exp) = p.to_integer().to_i32() {
                    if b.is_zero() {
                        if exp < 0 {
                            return Self::Float(f64::INFINITY);
                        } else if exp == 0 {
                            return Self::from_i64(1);
                        } else {
                            return Self::from_i64(0);
                        }
                    }
                    if exp.abs() <= 100_000 {
                        let exp_u = exp.unsigned_abs();
                        let numer = b.numer().pow(exp_u);
                        let denom = b.denom().pow(exp_u);
                        let res = BigRational::new(numer, denom);
                        if exp >= 0 {
                            return Self::Exact(res);
                        } else {
                            return Self::Exact(res.recip());
                        }
                    }
                }
                Self::Float(b.to_f64().unwrap_or(0.0).powf(p.to_f64().unwrap_or(0.0)))
            }
            (Self::Uncertainty { nominal, error }, p) => {
                let p_f = p.to_f64();
                let new_nom = nominal.powf(p_f);
                let new_err = (p_f * nominal.powf(p_f - 1.0) * error).abs();
                Self::Uncertainty {
                    nominal: new_nom,
                    error: new_err,
                }
            }
            (Self::Complex(a), p) => {
                let p_c = match p {
                    Self::Complex(c) => *c,
                    _ => Complex64::new(p.to_f64(), 0.0),
                };
                Self::Complex(a.powc(p_c))
            }
            (a, Self::Complex(p_c)) => {
                let a_c = Complex64::new(a.to_f64(), 0.0);
                Self::Complex(a_c.powc(*p_c))
            }
            (a, b) => {
                let af = a.to_f64();
                let bf = b.to_f64();
                if af < 0.0 && bf.fract() != 0.0 {
                    let a_c = Complex64::new(af, 0.0);
                    let b_c = Complex64::new(bf, 0.0);
                    let res = a_c.powc(b_c);
                    if res.im.abs() < 1e-12 {
                        Self::Float(res.re)
                    } else {
                        Self::Complex(res)
                    }
                } else {
                    Self::Float(af.powf(bf))
                }
            }
        }
    }

    pub fn sqrt(&self) -> Self {
        match self {
            Self::Exact(r) => {
                let f = r.to_f64().unwrap_or(f64::NAN);
                if f < 0.0 {
                    Self::Complex(Complex64::new(0.0, (-f).sqrt()))
                } else {
                    Self::Float(f.sqrt())
                }
            }
            Self::Float(f) => {
                if *f < 0.0 {
                    Self::Complex(Complex64::new(0.0, (-f).sqrt()))
                } else {
                    Self::Float(f.sqrt())
                }
            }
            Self::Uncertainty { nominal, error } => {
                let nom = nominal.sqrt();
                let err = (error / (2.0 * nom)).abs();
                Self::Uncertainty {
                    nominal: nom,
                    error: err,
                }
            }
            Self::Complex(c) => Self::Complex(c.sqrt()),
        }
    }

    pub fn sin(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.sin()),
            _ => Self::Float(self.to_f64().sin()),
        }
    }

    pub fn cos(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.cos()),
            _ => Self::Float(self.to_f64().cos()),
        }
    }

    pub fn tan(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.tan()),
            _ => Self::Float(self.to_f64().tan()),
        }
    }

    pub fn asin(&self) -> Self {
        Self::Float(self.to_f64().asin())
    }

    pub fn acos(&self) -> Self {
        Self::Float(self.to_f64().acos())
    }

    pub fn atan(&self) -> Self {
        Self::Float(self.to_f64().atan())
    }

    pub fn ln(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.ln()),
            _ => {
                let f = self.to_f64();
                if f <= 0.0 {
                    Self::Complex(Complex64::new(f, 0.0).ln())
                } else {
                    Self::Float(f.ln())
                }
            }
        }
    }

    pub fn log10(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.ln() / std::f64::consts::LN_10),
            _ => {
                let f = self.to_f64();
                if f < 0.0 {
                    Self::Complex(Complex64::new(f, 0.0).ln() / std::f64::consts::LN_10)
                } else {
                    Self::Float(f.log10())
                }
            }
        }
    }

    pub fn exp(&self) -> Self {
        match self {
            Self::Complex(c) => Self::Complex(c.exp()),
            _ => Self::Float(self.to_f64().exp()),
        }
    }

    pub fn abs(&self) -> Self {
        match self {
            Self::Exact(r) => Self::Exact(r.abs()),
            Self::Float(f) => Self::Float(f.abs()),
            Self::Uncertainty { nominal, error } => Self::Uncertainty {
                nominal: nominal.abs(),
                error: *error,
            },
            Self::Complex(c) => Self::Float(c.norm()),
        }
    }

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
                    format!("{:.10}", f).trim_end_matches('0').trim_end_matches('.').to_string()
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
                        format!("{:.10}", c.re).trim_end_matches('0').trim_end_matches('.').to_string()
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
                            format!("{:.10}", c.im).trim_end_matches('0').trim_end_matches('.').to_string()
                        };
                        format!("{}i", im_str)
                    }
                } else {
                    let re_str = if c.re.fract().abs() < 1e-10 {
                        format!("{:.0}", c.re.round())
                    } else {
                        format!("{:.10}", c.re).trim_end_matches('0').trim_end_matches('.').to_string()
                    };
                    let im_abs = c.im.abs();
                    let im_str = if (im_abs - 1.0).abs() < 1e-10 {
                        "i".to_string()
                    } else if im_abs.fract().abs() < 1e-10 {
                        format!("{:.0}i", im_abs.round())
                    } else {
                        format!("{}i", format!("{:.10}", im_abs).trim_end_matches('0').trim_end_matches('.'))
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

impl Add for NumericValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) => Self::Exact(a + b),
            (Self::Uncertainty { nominal: a, error: da }, Self::Uncertainty { nominal: b, error: db }) => {
                Self::Uncertainty {
                    nominal: a + b,
                    error: (da.powi(2) + db.powi(2)).sqrt(),
                }
            }
            (Self::Uncertainty { nominal: a, error: da }, b) => {
                Self::Uncertainty {
                    nominal: a + b.to_f64(),
                    error: da,
                }
            }
            (a, Self::Uncertainty { nominal: b, error: db }) => {
                Self::Uncertainty {
                    nominal: a.to_f64() + b,
                    error: db,
                }
            }
            (Self::Complex(a), Self::Complex(b)) => Self::Complex(a + b),
            (Self::Complex(a), b) => Self::Complex(a + Complex64::new(b.to_f64(), 0.0)),
            (a, Self::Complex(b)) => Self::Complex(Complex64::new(a.to_f64(), 0.0) + b),
            (a, b) => Self::Float(a.to_f64() + b.to_f64()),
        }
    }
}

impl Sub for NumericValue {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) => Self::Exact(a - b),
            (Self::Uncertainty { nominal: a, error: da }, Self::Uncertainty { nominal: b, error: db }) => {
                Self::Uncertainty {
                    nominal: a - b,
                    error: (da.powi(2) + db.powi(2)).sqrt(),
                }
            }
            (Self::Uncertainty { nominal: a, error: da }, b) => {
                Self::Uncertainty {
                    nominal: a - b.to_f64(),
                    error: da,
                }
            }
            (a, Self::Uncertainty { nominal: b, error: db }) => {
                Self::Uncertainty {
                    nominal: a.to_f64() - b,
                    error: db,
                }
            }
            (Self::Complex(a), Self::Complex(b)) => Self::Complex(a - b),
            (Self::Complex(a), b) => Self::Complex(a - Complex64::new(b.to_f64(), 0.0)),
            (a, Self::Complex(b)) => Self::Complex(Complex64::new(a.to_f64(), 0.0) - b),
            (a, b) => Self::Float(a.to_f64() - b.to_f64()),
        }
    }
}

impl Mul for NumericValue {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) => Self::Exact(a * b),
            (Self::Uncertainty { nominal: a, error: da }, Self::Uncertainty { nominal: b, error: db }) => {
                let prod = a * b;
                let err = ((b * da).powi(2) + (a * db).powi(2)).sqrt();
                Self::Uncertainty {
                    nominal: prod,
                    error: err,
                }
            }
            (Self::Uncertainty { nominal: a, error: da }, b) => {
                let bf = b.to_f64();
                Self::Uncertainty {
                    nominal: a * bf,
                    error: (da * bf).abs(),
                }
            }
            (a, Self::Uncertainty { nominal: b, error: db }) => {
                let af = a.to_f64();
                Self::Uncertainty {
                    nominal: af * b,
                    error: (db * af).abs(),
                }
            }
            (Self::Complex(a), Self::Complex(b)) => Self::Complex(a * b),
            (Self::Complex(a), b) => Self::Complex(a * Complex64::new(b.to_f64(), 0.0)),
            (a, Self::Complex(b)) => Self::Complex(Complex64::new(a.to_f64(), 0.0) * b),
            (a, b) => Self::Float(a.to_f64() * b.to_f64()),
        }
    }
}

impl Div for NumericValue {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) if !b.is_zero() => Self::Exact(a / b),
            (Self::Uncertainty { nominal: a, error: da }, Self::Uncertainty { nominal: b, error: db }) => {
                let quot = a / b;
                let err = if b != 0.0 {
                    ((da / b).powi(2) + ((a * db) / (b * b)).powi(2)).sqrt()
                } else {
                    f64::NAN
                };
                Self::Uncertainty {
                    nominal: quot,
                    error: err,
                }
            }
            (Self::Uncertainty { nominal: a, error: da }, b) => {
                let bf = b.to_f64();
                Self::Uncertainty {
                    nominal: a / bf,
                    error: (da / bf).abs(),
                }
            }
            (a, Self::Uncertainty { nominal: b, error: db }) => {
                let af = a.to_f64();
                let quot = af / b;
                let err = ((af * db) / (b * b)).abs();
                Self::Uncertainty {
                    nominal: quot,
                    error: err,
                }
            }
            (Self::Complex(a), Self::Complex(b)) => Self::Complex(a / b),
            (Self::Complex(a), b) => Self::Complex(a / Complex64::new(b.to_f64(), 0.0)),
            (a, Self::Complex(b)) => Self::Complex(Complex64::new(a.to_f64(), 0.0) / b),
            (a, b) => Self::Float(a.to_f64() / b.to_f64()),
        }
    }
}

impl Rem for NumericValue {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) => Self::Exact(a % b),
            (a, b) => Self::Float(a.to_f64() % b.to_f64()),
        }
    }
}

impl Neg for NumericValue {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Self::Exact(r) => Self::Exact(-r),
            Self::Float(f) => Self::Float(-f),
            Self::Uncertainty { nominal, error } => Self::Uncertainty {
                nominal: -nominal,
                error,
            },
            Self::Complex(c) => Self::Complex(-c),
        }
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_with_base(10, false))
    }
}
