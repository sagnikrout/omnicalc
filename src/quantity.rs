use crate::dimension::Dimension;
use crate::numeric::NumericValue;
use crate::units::UnitRegistry;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value: NumericValue,
    pub dimension: Dimension,
    pub unit_str: Option<String>,
}

impl Quantity {
    pub fn dimensionless(value: NumericValue) -> Self {
        Self {
            value,
            dimension: Dimension::DIMENSIONLESS,
            unit_str: None,
        }
    }

    pub fn from_unit(value: NumericValue, unit_name: &str, registry: &UnitRegistry) -> Result<Self, String> {
        let def = registry.lookup(unit_name)
            .ok_or_else(|| format!("Unknown unit: '{}'", unit_name))?;

        // Base SI value calculation
        let raw_f = value.to_f64();
        let base_f = (raw_f + def.offset) * def.scale;
        
        let val = match value {
            NumericValue::Uncertainty { error, .. } => {
                NumericValue::Uncertainty {
                    nominal: base_f,
                    error: error * def.scale,
                }
            }
            NumericValue::Exact(_) if def.offset == 0.0 && (def.scale.fract() == 0.0 || (1.0 / def.scale).fract() == 0.0) => {
                value * NumericValue::from_f64(def.scale)
            }
            _ => NumericValue::from_f64(base_f),
        };

        Ok(Self {
            value: val,
            dimension: def.dimension,
            unit_str: Some(def.symbol),
        })
    }

    pub fn to_unit(&self, target_unit_name: &str, registry: &UnitRegistry) -> Result<Self, String> {
        let target = registry.lookup(target_unit_name)
            .ok_or_else(|| format!("Unknown target unit: '{}'", target_unit_name))?;

        if self.dimension != target.dimension {
            return Err(format!(
                "Dimensional mismatch: cannot convert {} ({}) to '{}' ({})",
                self.format_plain(), self.dimension, target_unit_name, target.dimension
            ));
        }

        let base_f = self.value.to_f64();
        let target_f = (base_f / target.scale) - target.offset;

        let res_val = match &self.value {
            NumericValue::Uncertainty { error, .. } => {
                NumericValue::Uncertainty {
                    nominal: target_f,
                    error: error / target.scale,
                }
            }
            _ => NumericValue::from_f64(target_f),
        };

        Ok(Self {
            value: res_val,
            dimension: target.dimension,
            unit_str: Some(target.symbol),
        })
    }

    pub fn format_plain(&self) -> String {
        match &self.unit_str {
            Some(u) => format!("{} {}", self.value, u),
            None => format!("{}", self.value),
        }
    }

    pub fn format_with_registry(&self, registry: &UnitRegistry) -> String {
        if let Some(ref u) = self.unit_str {
            if let Some(def) = registry.lookup(u)
                && (def.offset != 0.0 || (def.scale != 1.0 && def.scale > 0.0)) {
                    let base_f = self.value.to_f64();
                    let display_f = (base_f / def.scale) - def.offset;
                    let display_val = match &self.value {
                        NumericValue::Uncertainty { error, .. } => NumericValue::Uncertainty {
                            nominal: display_f,
                            error: error / def.scale,
                        },
                        _ => NumericValue::from_f64(display_f),
                    };
                    return format!("{} {}", display_val, u);
                }
            format!("{} {}", self.value, u)
        } else if self.dimension.is_dimensionless() {
            format!("{}", self.value)
        } else {
            let unit_name = if self.dimension == Dimension::LENGTH {
                "m"
            } else if self.dimension == Dimension::MASS {
                "kg"
            } else if self.dimension == Dimension::TIME {
                "s"
            } else if self.dimension == Dimension::CURRENT {
                "A"
            } else if self.dimension == Dimension::TEMPERATURE {
                "K"
            } else if self.dimension == Dimension::VELOCITY {
                "m/s"
            } else if self.dimension == Dimension::FORCE {
                "N"
            } else if self.dimension == Dimension::ENERGY {
                "J"
            } else if self.dimension == Dimension::POWER {
                "W"
            } else if self.dimension == Dimension::PRESSURE {
                "Pa"
            } else if self.dimension == Dimension::DIGITAL {
                "bit"
            } else if self.dimension == Dimension::CURRENCY {
                "USD"
            } else if self.dimension == Dimension::LENGTH.pow(2) {
                "m^2"
            } else if self.dimension == Dimension::LENGTH.pow(3) {
                "m^3"
            } else {
                return format!("{} [{}]", self.value, self.dimension);
            };
            format!("{} {}", self.value, unit_name)
        }
    }

    pub fn format_raw(&self) -> String {
        self.value.format_with_base(10, true).to_string()
    }
}

impl Add for Quantity {
    type Output = Result<Quantity, String>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.dimension != rhs.dimension {
            return Err(format!(
                "Type error: Dimensional mismatch in addition ({} vs {})",
                self.dimension, rhs.dimension
            ));
        }
        Ok(Quantity {
            value: self.value + rhs.value,
            dimension: self.dimension,
            unit_str: self.unit_str.or(rhs.unit_str),
        })
    }
}

impl Sub for Quantity {
    type Output = Result<Quantity, String>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.dimension != rhs.dimension {
            return Err(format!(
                "Type error: Dimensional mismatch in subtraction ({} vs {})",
                self.dimension, rhs.dimension
            ));
        }
        Ok(Quantity {
            value: self.value - rhs.value,
            dimension: self.dimension,
            unit_str: self.unit_str.or(rhs.unit_str),
        })
    }
}

impl Mul for Quantity {
    type Output = Quantity;
    fn mul(self, rhs: Self) -> Self::Output {
        let new_dim = self.dimension + rhs.dimension;
        Quantity {
            value: self.value * rhs.value,
            dimension: new_dim,
            unit_str: None, // needs formatting simplification
        }
    }
}

impl Div for Quantity {
    type Output = Quantity;
    fn div(self, rhs: Self) -> Self::Output {
        let new_dim = self.dimension - rhs.dimension;
        Quantity {
            value: self.value / rhs.value,
            dimension: new_dim,
            unit_str: None,
        }
    }
}

impl Neg for Quantity {
    type Output = Quantity;
    fn neg(self) -> Self::Output {
        Quantity {
            value: -self.value,
            dimension: self.dimension,
            unit_str: self.unit_str,
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref u) = self.unit_str {
            write!(f, "{} {}", self.value, u)
        } else if self.dimension.is_dimensionless() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "{} [{}]", self.value, self.dimension)
        }
    }
}
