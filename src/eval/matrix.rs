#![allow(clippy::needless_range_loop)]

use super::la::LaValue;

fn validate_matrix(
    m: &[Vec<f64>],
    require_square: bool,
    op_name: &str,
) -> Result<(usize, usize), String> {
    let rows = m.len();
    if rows == 0 || m[0].is_empty() {
        return Err(format!("Cannot perform {} on empty matrix", op_name));
    }
    let cols = m[0].len();
    for row in m {
        if row.len() != cols {
            return Err("Matrix row length mismatch".to_string());
        }
        if row.iter().any(|x| !x.is_finite()) {
            return Err(format!("Matrix contains non-finite values in {}", op_name));
        }
    }
    if require_square && rows != cols {
        return Err(format!(
            "{} requires a square matrix (got {}x{})",
            op_name, rows, cols
        ));
    }
    Ok((rows, cols))
}

pub fn eval_transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if m.is_empty() || m[0].is_empty() {
        return Vec::new();
    }
    let rows = m.len();
    let cols = m[0].len();
    let mut res = vec![vec![0.0; rows]; cols];
    for i in 0..rows {
        for j in 0..cols {
            res[j][i] = m[i][j];
        }
    }
    res
}

pub fn eval_det(m: &[Vec<f64>]) -> Result<f64, String> {
    let (n, _) = validate_matrix(m, true, "determinant")?;
    if n == 1 {
        return Ok(m[0][0]);
    }
    if n == 2 {
        let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
        if !det.is_finite() {
            return Err("Determinant overflowed to non-finite value".to_string());
        }
        return Ok(det);
    }
    let mut a: Vec<Vec<f64>> = m.to_vec();
    let mut det = 1.0;
    let mut swaps = 0;
    for i in 0..n {
        let mut max_row = i;
        let mut max_val = a[i][i].abs();
        for k in i + 1..n {
            if a[k][i].abs() > max_val {
                max_val = a[k][i].abs();
                max_row = k;
            }
        }
        if !max_val.is_finite() {
            return Err("Determinant overflowed to non-finite value".to_string());
        }
        if max_val < 1e-12 {
            return Ok(0.0);
        }
        if max_row != i {
            a.swap(i, max_row);
            swaps += 1;
        }
        let pivot = a[i][i];
        det *= pivot;
        for j in i + 1..n {
            let factor = a[j][i] / pivot;
            for k in i + 1..n {
                a[j][k] -= factor * a[i][k];
            }
        }
    }
    if swaps % 2 == 1 {
        det = -det;
    }
    if !det.is_finite() {
        return Err("Determinant overflowed to non-finite value".to_string());
    }
    Ok(det)
}

pub fn eval_inv(m: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
    let (n, _) = validate_matrix(m, true, "matrix inversion")?;
    let mut a: Vec<Vec<f64>> = m.to_vec();
    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        inv[i][i] = 1.0;
    }
    for i in 0..n {
        let mut max_row = i;
        let mut max_val = a[i][i].abs();
        for k in i + 1..n {
            if a[k][i].abs() > max_val {
                max_val = a[k][i].abs();
                max_row = k;
            }
        }
        if max_val < 1e-12 {
            return Err("Matrix is singular (not invertible)".to_string());
        }
        if max_row != i {
            a.swap(i, max_row);
            inv.swap(i, max_row);
        }
        let pivot = a[i][i];
        for j in 0..n {
            a[i][j] /= pivot;
            inv[i][j] /= pivot;
        }
        for k in 0..n {
            if k != i {
                let factor = a[k][i];
                for j in 0..n {
                    a[k][j] -= factor * a[i][j];
                    inv[k][j] -= factor * inv[i][j];
                }
            }
        }
    }
    for row in &mut inv {
        for val in row {
            if val.abs() < 1e-12 {
                *val = 0.0;
            }
        }
    }
    Ok(inv)
}

pub fn eval_matrix_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
    let (m, k) = validate_matrix(a, false, "multiplication")?;
    let (p, n) = validate_matrix(b, false, "multiplication")?;
    if k != p {
        return Err(format!(
            "Matrix dimension mismatch in multiplication: left has {} columns, right has {} rows",
            k, p
        ));
    }
    let mut res = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for r in 0..k {
                sum += a[i][r] * b[r][j];
            }
            res[i][j] = sum;
        }
    }
    Ok(res)
}

pub fn la_add(a: LaValue, b: LaValue) -> Result<LaValue, String> {
    match (a, b) {
        (LaValue::Scalar(s1), LaValue::Scalar(s2)) => Ok(LaValue::Scalar(s1 + s2)),
        (LaValue::Vector(v1), LaValue::Vector(v2)) => {
            if v1.len() != v2.len() {
                return Err(format!(
                    "Vector length mismatch in addition: {} != {}",
                    v1.len(),
                    v2.len()
                ));
            }
            Ok(LaValue::Vector(
                v1.iter().zip(v2.iter()).map(|(x, y)| x + y).collect(),
            ))
        }
        (LaValue::Matrix(m1), LaValue::Matrix(m2)) => {
            if m1.len() != m2.len() || m1.is_empty() || m1[0].len() != m2[0].len() {
                return Err("Matrix dimension mismatch in addition".to_string());
            }
            let rows = m1.len();
            let cols = m1[0].len();
            let mut res = vec![vec![0.0; cols]; rows];
            for i in 0..rows {
                for j in 0..cols {
                    res[i][j] = m1[i][j] + m2[i][j];
                }
            }
            Ok(LaValue::Matrix(res))
        }
        _ => Err("Invalid operands for linear algebra addition".to_string()),
    }
}

pub fn la_sub(a: LaValue, b: LaValue) -> Result<LaValue, String> {
    match (a, b) {
        (LaValue::Scalar(s1), LaValue::Scalar(s2)) => Ok(LaValue::Scalar(s1 - s2)),
        (LaValue::Vector(v1), LaValue::Vector(v2)) => {
            if v1.len() != v2.len() {
                return Err(format!(
                    "Vector length mismatch in subtraction: {} != {}",
                    v1.len(),
                    v2.len()
                ));
            }
            Ok(LaValue::Vector(
                v1.iter().zip(v2.iter()).map(|(x, y)| x - y).collect(),
            ))
        }
        (LaValue::Matrix(m1), LaValue::Matrix(m2)) => {
            if m1.len() != m2.len() || m1.is_empty() || m1[0].len() != m2[0].len() {
                return Err("Matrix dimension mismatch in subtraction".to_string());
            }
            let rows = m1.len();
            let cols = m1[0].len();
            let mut res = vec![vec![0.0; cols]; rows];
            for i in 0..rows {
                for j in 0..cols {
                    res[i][j] = m1[i][j] - m2[i][j];
                }
            }
            Ok(LaValue::Matrix(res))
        }
        _ => Err("Invalid operands for linear algebra subtraction".to_string()),
    }
}

pub fn la_mul(a: LaValue, b: LaValue) -> Result<LaValue, String> {
    match (a, b) {
        (LaValue::Scalar(s1), LaValue::Scalar(s2)) => Ok(LaValue::Scalar(s1 * s2)),
        (LaValue::Scalar(s), LaValue::Vector(v)) | (LaValue::Vector(v), LaValue::Scalar(s)) => {
            Ok(LaValue::Vector(v.iter().map(|x| x * s).collect()))
        }
        (LaValue::Scalar(s), LaValue::Matrix(m)) | (LaValue::Matrix(m), LaValue::Scalar(s)) => {
            Ok(LaValue::Matrix(
                m.into_iter()
                    .map(|row| row.into_iter().map(|x| x * s).collect())
                    .collect(),
            ))
        }
        (LaValue::Matrix(m1), LaValue::Matrix(m2)) => {
            let res = eval_matrix_mul(&m1, &m2)?;
            Ok(LaValue::Matrix(res))
        }
        (LaValue::Matrix(m), LaValue::Vector(v)) => {
            if m.is_empty() || m[0].len() != v.len() {
                return Err(format!(
                    "Dimension mismatch: matrix has {} columns, vector has length {}",
                    if m.is_empty() { 0 } else { m[0].len() },
                    v.len()
                ));
            }
            let mut res = Vec::new();
            for row in m {
                res.push(row.iter().zip(v.iter()).map(|(x, y)| x * y).sum());
            }
            Ok(LaValue::Vector(res))
        }
        (LaValue::Vector(_), LaValue::Vector(_)) => {
            Err("Vector multiplication is ambiguous; use 'dot(u, v)' or 'cross(u, v)'".to_string())
        }
        _ => Err("Invalid operands for linear algebra multiplication".to_string()),
    }
}
