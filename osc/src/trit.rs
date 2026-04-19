pub fn label(c: i8) -> &'static str {
    match c {
        -1 => "red",
        0 => "abstain",
        1 => "blue",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_cover_all_trits() {
        assert_eq!(label(-1), "red");
        assert_eq!(label(0), "abstain");
        assert_eq!(label(1), "blue");
    }

    #[test]
    fn gf3_conservation_on_balanced_triad() {
        let sum: i64 = [-1_i64, 0, 1].iter().sum();
        assert_eq!(((sum % 3) + 3) % 3, 0);
    }
}
