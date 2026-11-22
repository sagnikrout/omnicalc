pub fn eval_dot(u: &[f64], v: &[f64]) -> Result<f64, String> {
    if u.len() != v.len() {
        return Err(format!(
            "Vector length mismatch in dot product: {} != {}",
            u.len(),
            v.len()
        ));
    }
    Ok(u.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
}

pub fn eval_cross(u: &[f64], v: &[f64]) -> Result<Vec<f64>, String> {
    if u.len() != 3 || v.len() != 3 {
        return Err(format!(
            "Cross product requires 3D vectors (got lengths {} and {})",
            u.len(),
            v.len()
        ));
    }
    Ok(vec![
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ])
}

pub fn eval_norm(u: &[f64]) -> Result<f64, String> {
    let sum_sq: f64 = u.iter().map(|x| x * x).sum();
    Ok(sum_sq.sqrt())
}
