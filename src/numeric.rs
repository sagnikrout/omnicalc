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
                    if exp >= 0 {
                        let mut res = BigRational::one();
                        for _ in 0..exp {
                            res *= b;
                        }
                        return Self::Exact(res);
                    } else {
                        let mut res = BigRational::one();
                        for _ in 0..(-exp) {
                            res *= b;
                        }
                        return Self::Exact(res.recip());
                    }
                }
                Self::Float(b.to_f64().unwrap().powf(p.to_f64().unwrap()))
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
            (a, b) => Self::Float(a.to_f64().powf(b.to_f64())),
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
        Self::Float(self.to_f64().sin())
    }

    pub fn cos(&self) -> Self {
        Self::Float(self.to_f64().cos())
    }

    pub fn tan(&self) -> Self {
        Self::Float(self.to_f64().tan())
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
        let f = self.to_f64();
        if f <= 0.0 {
            Self::Complex(Complex64::new(f, 0.0).ln())
        } else {
            Self::Float(f.ln())
        }
    }

    pub fn log10(&self) -> Self {
        Self::Float(self.to_f64().log10())
    }

    pub fn exp(&self) -> Self {
        Self::Float(self.to_f64().exp())
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
                } else if c.im.abs() < 1e-12 {
                    if c.re.fract() == 0.0 && c.re.abs() < 1e15 {
                        format!("{:.0}", c.re)
                    } else {
                        format!("{:.10}", c.re).trim_end_matches('0').trim_end_matches('.').to_string()
                    }
                } else if c.re.abs() < 1e-12 {
                    if (c.im - 1.0).abs() < 1e-12 {
                        "i".to_string()
                    } else if (c.im + 1.0).abs() < 1e-12 {
                        "-i".to_string()
                    } else {
                        format!("{}i", c.im)
                    }
                } else if c.im >= 0.0 {
                    format!("{} + {}i", c.re, c.im)
                } else {
                    format!("{} - {}i", c.re, -c.im)
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
                let rel_err = ((da / a).powi(2) + (db / b).powi(2)).sqrt();
                Self::Uncertainty {
                    nominal: prod,
                    error: (prod * rel_err).abs(),
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
                let rel_err = ((da / a).powi(2) + (db / b).powi(2)).sqrt();
                Self::Uncertainty {
                    nominal: quot,
                    error: (quot * rel_err).abs(),
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
                let rel_err = (db / b).abs();
                Self::Uncertainty {
                    nominal: quot,
                    error: (quot * rel_err).abs(),
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
