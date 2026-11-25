use crate::numeric::NumericValue;
use crate::quantity::Quantity;
use crate::units::UnitRegistry;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};

/// Standard error function approximation
pub(crate) fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation formula 7.1.26
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs();

    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    sign * y
}

pub(crate) fn gcd_bigint(mut a: BigInt, mut b: BigInt) -> BigInt {
    while !b.is_zero() {
        let t = b.clone();
        b = a % b;
        a = t;
    }
    a.abs()
}

pub(crate) fn lcm_bigint(a: BigInt, b: BigInt) -> BigInt {
    if a.is_zero() || b.is_zero() {
        BigInt::zero()
    } else {
        let g = gcd_bigint(a.clone(), b.clone());
        ((a / g) * b).abs()
    }
}

pub(crate) fn round_quantity(
    q: &Quantity,
    mode: &str,
    registry: &UnitRegistry,
) -> Result<Quantity, String> {
    let apply = |val: f64| match mode {
        "round" => val.round(),
        "floor" => val.floor(),
        _ => val.ceil(),
    };
    if let Some(ref unit_name) = q.unit_str
        && let Ok(converted) = q.to_unit(unit_name, registry)
    {
        let rounded = apply(converted.value.to_f64());
        return Quantity::from_unit(NumericValue::from_f64(rounded), unit_name, registry);
    }
    Ok(Quantity {
        value: NumericValue::from_f64(apply(q.value.to_f64())),
        dimension: q.dimension,
        unit_str: q.unit_str.clone(),
    })
}
