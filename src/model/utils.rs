pub fn default_block_size(file_size: u64) -> u32 {
    match file_size as u32 {
        0..=1_000_000 => 1024,           // 1KB for files <= 1MB
        1_000_001..=100_000_000 => 4096, // 4KB for files <= 100MB
        _ => 16384,                      // 16KB for larger files
    }
}

pub fn fit_into_power_of_two_u32(n: u32) -> u32 {
    if n < 1 {
        return 1;
    }

    let mut rounded_up = n;

    // -1 for cases where it is already a power of two and we dont wanna double it
    rounded_up -= 1;

    // sneaky bitwise hack to round up to the next power of two ;)
    rounded_up |= rounded_up >> 1;
    rounded_up |= rounded_up >> 2;
    rounded_up |= rounded_up >> 4;
    rounded_up |= rounded_up >> 8;
    rounded_up |= rounded_up >> 16;

    // add 1 to the rounded up value
    rounded_up += 1;

    let rounded_down = rounded_up >> 1;

    let midpoint = rounded_down + (rounded_up - rounded_down) / 2;

    if n < midpoint {
        rounded_down
    } else {
        rounded_up
    }
}

pub fn fit_into_power_of_two_u64(n: u64) -> u64 {
    if n < 1 {
        return 1;
    }

    let mut rounded_up = n;

    // -1 for cases where it is already a power of two and we dont wanna double it
    rounded_up -= 1;

    // sneaky bitwise hack to round up to the next power of two ;)
    rounded_up |= rounded_up >> 1;
    rounded_up |= rounded_up >> 2;
    rounded_up |= rounded_up >> 4;
    rounded_up |= rounded_up >> 8;
    rounded_up |= rounded_up >> 16;
    rounded_up |= rounded_up >> 32;

    // add 1 to the rounded up value
    rounded_up += 1;

    let rounded_down = rounded_up >> 1;

    let midpoint = rounded_down + (rounded_up - rounded_down) / 2;

    if n < midpoint {
        rounded_down
    } else {
        rounded_up
    }
}
