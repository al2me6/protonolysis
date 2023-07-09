use std::f64::consts::PI;

pub mod distribution;
pub mod error_function;
pub mod gaussian_sum;

pub fn pascals_triangle(n: u32) -> impl Iterator<Item = u32> {
    let mut prev = 1;
    (0..=n).map(move |k| {
        if k > 0 {
            prev = prev * (n + 1 - k) / k;
        }
        prev
    })
}

/// The `n`-th row of Pascal's triangle, with the sum of the row normalized to unity.
pub fn normalized_pascals_triangle(n: u32) -> impl Iterator<Item = f64> {
    // Sum of (0-indexed) n-th row is 2^n.
    let row_sum = f64::from(1 << n);
    pascals_triangle(n).map(move |a| f64::from(a) / row_sum)
}

#[must_use]
pub fn ease_transition(factor: f64) -> f64 {
    0.5 * (1.0 - (PI * factor).cos())
}

#[must_use]
pub fn ease_transition_inverse(value: f64) -> f64 {
    (1.0 - 2.0 * value).acos() / PI
}

#[must_use]
/// All hail negative zero.
pub fn negate_nonzero(x: f64) -> f64 {
    if x.abs() == 0.0 {
        0.0
    } else {
        -x
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    #[test]
    fn pascals_triangle() {
        const EXPECTED: [&[u32]; 7] = [
            &[1],
            &[1, 1],
            &[1, 2, 1],
            &[1, 3, 3, 1],
            &[1, 4, 6, 4, 1],
            &[1, 5, 10, 10, 5, 1],
            &[1, 6, 15, 20, 15, 6, 1],
        ];
        for (n, &expected) in EXPECTED.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let computed = super::pascals_triangle(n as _).collect_vec();
            assert!(computed == expected);
        }
    }

    #[test]
    fn normalized_pascals_triangle() {
        for n in 0..=6 {
            let sum = super::normalized_pascals_triangle(n).sum::<f64>();
            approx::assert_abs_diff_eq!(sum, 1.);
        }
    }
}
