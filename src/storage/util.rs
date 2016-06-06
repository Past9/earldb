

pub fn is_power_of_two(n: usize) -> bool {
    return (n != 0) && (n & (n - 1)) == 0;
}
