use num_bigint::BigInt;
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};

pub mod format;
pub mod math;
pub mod ops;

#[allow(unused_imports)]
pub use format::*;
#[allow(unused_imports)]
pub use math::*;
#[allow(unused_imports)]
pub use ops::*;

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

    pub fn from_bigint(b: BigInt) -> Self {
        Self::Exact(BigRational::from_integer(b))
    }

    pub fn to_bigint(&self) -> Option<BigInt> {
        match self {
            Self::Exact(r) => {
                if r.is_integer() {
                    Some(r.to_integer())
                } else {
                    r.to_f64().map(|f| BigInt::from(f as i64))
                }
            }
            Self::Float(f) => Some(BigInt::from(*f as i64)),
            _ => None,
        }
    }

    pub fn to_exact_bigint(&self) -> Option<BigInt> {
        match self {
            Self::Exact(r) => {
                if r.is_integer() {
                    Some(r.to_integer())
                } else {
                    None
                }
            }
            Self::Float(f) => {
                if f.is_finite()
                    && f.fract() == 0.0
                    && *f >= (i64::MIN as f64)
                    && *f <= (i64::MAX as f64)
                {
                    Some(BigInt::from(*f as i64))
                } else {
                    None
                }
            }
            _ => None,
        }
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
}
