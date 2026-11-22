use num_complex::Complex64;
use num_traits::Zero;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use super::NumericValue;

impl Add for NumericValue {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Exact(a), Self::Exact(b)) => Self::Exact(a + b),
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => Self::Uncertainty {
                nominal: a + b,
                error: (da.powi(2) + db.powi(2)).sqrt(),
            },
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                b,
            ) => Self::Uncertainty {
                nominal: a + b.to_f64(),
                error: da,
            },
            (
                a,
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => Self::Uncertainty {
                nominal: a.to_f64() + b,
                error: db,
            },
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
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => Self::Uncertainty {
                nominal: a - b,
                error: (da.powi(2) + db.powi(2)).sqrt(),
            },
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                b,
            ) => Self::Uncertainty {
                nominal: a - b.to_f64(),
                error: da,
            },
            (
                a,
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => Self::Uncertainty {
                nominal: a.to_f64() - b,
                error: db,
            },
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
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => {
                let prod = a * b;
                let err = ((b * da).powi(2) + (a * db).powi(2)).sqrt();
                Self::Uncertainty {
                    nominal: prod,
                    error: err,
                }
            }
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                b,
            ) => {
                let bf = b.to_f64();
                Self::Uncertainty {
                    nominal: a * bf,
                    error: (da * bf).abs(),
                }
            }
            (
                a,
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => {
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
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => {
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
            (
                Self::Uncertainty {
                    nominal: a,
                    error: da,
                },
                b,
            ) => {
                let bf = b.to_f64();
                Self::Uncertainty {
                    nominal: a / bf,
                    error: (da / bf).abs(),
                }
            }
            (
                a,
                Self::Uncertainty {
                    nominal: b,
                    error: db,
                },
            ) => {
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
