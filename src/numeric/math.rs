use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};

use super::NumericValue;

impl NumericValue {
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

    pub fn cbrt(&self) -> Self {
        match self {
            Self::Exact(r) => {
                let f = r.to_f64().unwrap_or(0.0).cbrt();
                if (f.round().powi(3) - r.to_f64().unwrap_or(0.0)).abs() < 1e-10 {
                    Self::from_i64(f.round() as i64)
                } else {
                    Self::Float(f)
                }
            }
            Self::Float(f) => Self::Float(f.cbrt()),
            Self::Uncertainty { nominal, error } => {
                let nom = nominal.cbrt();
                let err = (error / (3.0 * nom.powi(2))).abs();
                Self::Uncertainty {
                    nominal: nom,
                    error: err,
                }
            }
            Self::Complex(c) => Self::Complex(c.powf(1.0 / 3.0)),
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

    pub fn log2(&self) -> Self {
        match self {
            Self::Exact(r) if r.is_positive() => {
                let f = r.to_f64().unwrap_or(0.0).log2();
                if (f.round() - f).abs() < 1e-10 {
                    Self::from_i64(f.round() as i64)
                } else {
                    Self::Float(f)
                }
            }
            Self::Complex(c) => Self::Complex(c.ln() / std::f64::consts::LN_2),
            _ => {
                let f = self.to_f64();
                if f < 0.0 {
                    Self::Complex(Complex64::new(f, 0.0).ln() / std::f64::consts::LN_2)
                } else {
                    Self::Float(f.log2())
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
}
