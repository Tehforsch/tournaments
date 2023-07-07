fn binomial(n: usize, k: usize) -> usize {
    ((n - k + 1)..=n).product::<usize>() / (1..=k).product::<usize>()
}

pub fn binomial_distribution(p: f64, n: usize, k: usize) -> f64 {
    binomial(n, k) as f64 * p.powi((n - k) as i32) * (1.0 - p).powi(k as i32)
}

#[cfg(test)]
mod tests {
    #[test]
    fn binomial() {
        use super::binomial;
        assert_eq!(binomial(0, 0), 1);
        assert_eq!(binomial(1, 0), 1);
        assert_eq!(binomial(1, 1), 1);
        assert_eq!(binomial(2, 0), 1);
        assert_eq!(binomial(2, 1), 2);
        assert_eq!(binomial(2, 2), 1);
        assert_eq!(binomial(3, 0), 1);
        assert_eq!(binomial(3, 1), 3);
        assert_eq!(binomial(3, 2), 3);
        assert_eq!(binomial(3, 3), 1);
    }

    #[test]
    fn binomial_distribution() {
        use super::binomial_distribution;
        assert_eq!(
            binomial_distribution(0.5, 3, 2) + binomial_distribution(0.5, 3, 3),
            0.5
        )
    }
}
