use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// Standard physical dimensions:
/// Length (L), Mass (M), Time (T), Electric Current (I),
/// Temperature (Θ), Amount of Substance (N), Luminous Intensity (J),
/// Digital Storage (D), Currency (C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Dimension {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub current: i8,
    pub temperature: i8,
    pub amount: i8,
    pub luminosity: i8,
    pub digital: i8,
    pub currency: i8,
}

impl Dimension {
    pub const DIMENSIONLESS: Self = Self {
        length: 0,
        mass: 0,
        time: 0,
        current: 0,
        temperature: 0,
        amount: 0,
        luminosity: 0,
        digital: 0,
        currency: 0,
    };

    pub const LENGTH: Self = Self {
        length: 1,
        ..Self::DIMENSIONLESS
    };
    pub const MASS: Self = Self {
        mass: 1,
        ..Self::DIMENSIONLESS
    };
    pub const TIME: Self = Self {
        time: 1,
        ..Self::DIMENSIONLESS
    };
    pub const CURRENT: Self = Self {
        current: 1,
        ..Self::DIMENSIONLESS
    };
    pub const TEMPERATURE: Self = Self {
        temperature: 1,
        ..Self::DIMENSIONLESS
    };
    pub const AMOUNT: Self = Self {
        amount: 1,
        ..Self::DIMENSIONLESS
    };
    pub const LUMINOSITY: Self = Self {
        luminosity: 1,
        ..Self::DIMENSIONLESS
    };
    pub const DIGITAL: Self = Self {
        digital: 1,
        ..Self::DIMENSIONLESS
    };
    pub const CURRENCY: Self = Self {
        currency: 1,
        ..Self::DIMENSIONLESS
    };

    pub const VELOCITY: Self = Self {
        length: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };
    pub const ACCELERATION: Self = Self {
        length: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    pub const FORCE: Self = Self {
        length: 1,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    pub const ENERGY: Self = Self {
        length: 2,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    pub const POWER: Self = Self {
        length: 2,
        mass: 1,
        time: -3,
        ..Self::DIMENSIONLESS
    };
    pub const PRESSURE: Self = Self {
        length: -1,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    pub const VOLTAGE: Self = Self {
        length: 2,
        mass: 1,
        time: -3,
        current: -1,
        ..Self::DIMENSIONLESS
    };
    pub const RESISTANCE: Self = Self {
        length: 2,
        mass: 1,
        time: -3,
        current: -2,
        ..Self::DIMENSIONLESS
    };
    pub const CHARGE: Self = Self {
        time: 1,
        current: 1,
        ..Self::DIMENSIONLESS
    };
    pub const FREQUENCY: Self = Self {
        time: -1,
        ..Self::DIMENSIONLESS
    };
    pub const DATA_RATE: Self = Self {
        digital: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };

    pub fn is_dimensionless(&self) -> bool {
        *self == Self::DIMENSIONLESS
    }

    pub fn pow(&self, power: i8) -> Self {
        Self {
            length: self.length * power,
            mass: self.mass * power,
            time: self.time * power,
            current: self.current * power,
            temperature: self.temperature * power,
            amount: self.amount * power,
            luminosity: self.luminosity * power,
            digital: self.digital * power,
            currency: self.currency * power,
        }
    }

    pub fn is_divisible_by(&self, n: i8) -> bool {
        self.length % n == 0
            && self.mass % n == 0
            && self.time % n == 0
            && self.current % n == 0
            && self.temperature % n == 0
            && self.amount % n == 0
            && self.luminosity % n == 0
            && self.digital % n == 0
            && self.currency % n == 0
    }

    pub fn div_scalar(&self, n: i8) -> Self {
        Self {
            length: self.length / n,
            mass: self.mass / n,
            time: self.time / n,
            current: self.current / n,
            temperature: self.temperature / n,
            amount: self.amount / n,
            luminosity: self.luminosity / n,
            digital: self.digital / n,
            currency: self.currency / n,
        }
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self {
            length: self.length.checked_add(rhs.length)?,
            mass: self.mass.checked_add(rhs.mass)?,
            time: self.time.checked_add(rhs.time)?,
            current: self.current.checked_add(rhs.current)?,
            temperature: self.temperature.checked_add(rhs.temperature)?,
            amount: self.amount.checked_add(rhs.amount)?,
            luminosity: self.luminosity.checked_add(rhs.luminosity)?,
            digital: self.digital.checked_add(rhs.digital)?,
            currency: self.currency.checked_add(rhs.currency)?,
        })
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self {
            length: self.length.checked_sub(rhs.length)?,
            mass: self.mass.checked_sub(rhs.mass)?,
            time: self.time.checked_sub(rhs.time)?,
            current: self.current.checked_sub(rhs.current)?,
            temperature: self.temperature.checked_sub(rhs.temperature)?,
            amount: self.amount.checked_sub(rhs.amount)?,
            luminosity: self.luminosity.checked_sub(rhs.luminosity)?,
            digital: self.digital.checked_sub(rhs.digital)?,
            currency: self.currency.checked_sub(rhs.currency)?,
        })
    }
}

impl Add for Dimension {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            length: self.length + rhs.length,
            mass: self.mass + rhs.mass,
            time: self.time + rhs.time,
            current: self.current + rhs.current,
            temperature: self.temperature + rhs.temperature,
            amount: self.amount + rhs.amount,
            luminosity: self.luminosity + rhs.luminosity,
            digital: self.digital + rhs.digital,
            currency: self.currency + rhs.currency,
        }
    }
}

impl Sub for Dimension {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            length: self.length - rhs.length,
            mass: self.mass - rhs.mass,
            time: self.time - rhs.time,
            current: self.current - rhs.current,
            temperature: self.temperature - rhs.temperature,
            amount: self.amount - rhs.amount,
            luminosity: self.luminosity - rhs.luminosity,
            digital: self.digital - rhs.digital,
            currency: self.currency - rhs.currency,
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul for Dimension {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self + rhs
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Div for Dimension {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self - rhs
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return write!(f, "1");
        }
        let dims = [
            ("L", self.length),
            ("M", self.mass),
            ("T", self.time),
            ("I", self.current),
            ("Θ", self.temperature),
            ("N", self.amount),
            ("J", self.luminosity),
            ("D", self.digital),
            ("C", self.currency),
        ];
        let mut parts = Vec::new();
        for (name, exp) in dims {
            if exp == 1 {
                parts.push(name.to_string());
            } else if exp != 0 {
                parts.push(format!("{}^{}", name, exp));
            }
        }
        write!(f, "{}", parts.join("·"))
    }
}
