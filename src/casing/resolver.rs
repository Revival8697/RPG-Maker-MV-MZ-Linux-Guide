use std::collections::HashMap;

pub fn select_canonical_stem(casing_counts: &HashMap<String, usize>) -> String {
    let mut sorted_stems: Vec<&String> = casing_counts.keys().collect();
    sorted_stems.sort_by(|a, b| {
        let count_a = casing_counts[*a];
        let count_b = casing_counts[*b];
        count_b
            .cmp(&count_a)
            .then_with(|| {
                let u_a = a.chars().filter(|c| c.is_uppercase()).count();
                let u_b = b.chars().filter(|c| c.is_uppercase()).count();
                u_b.cmp(&u_a)
            })
            .then_with(|| a.cmp(b))
    });
    sorted_stems[0].clone()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_canonical_stem_by_frequency() {
        let mut map = HashMap::new();
        map.insert("inside_a2".to_string(), 1);
        map.insert("Inside_A2".to_string(), 5);

        assert_eq!(select_canonical_stem(&map), "Inside_A2");
    }

    #[test]
    fn test_select_canonical_stem_tie_breaker_uppercase() {
        let mut map = HashMap::new();
        map.insert("inside_a2".to_string(), 2);
        map.insert("Inside_A2".to_string(), 2);

        assert_eq!(select_canonical_stem(&map), "Inside_A2");
    }
}
