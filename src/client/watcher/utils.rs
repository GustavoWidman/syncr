pub enum SplitResult {
    Positive(usize),
    Negative,
}

pub fn split_i32(value: i32) -> SplitResult {
    match value {
        n if n >= 0 => SplitResult::Positive(n.try_into().unwrap()),
        _ => SplitResult::Negative,
    }
}
