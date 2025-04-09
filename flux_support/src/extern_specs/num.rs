// The spec we need to prove log_base_two
#[flux_rs::extern_spec]
impl u32 {
    #[sig(fn(num: u32) -> u32{r: (num == 0 => r == 32) &&
                                 (num > 0 => r <= 31) &&
                                 (num > 1 => r <= 30)
                            })]
    fn leading_zeros(self) -> u32;

    #[sig(fn(num: u32) -> u32{r: (num == 0 => r == 32) && (num != 0 =>r <= 31) })]
    fn trailing_zeros(self) -> u32;
}

#[flux_rs::extern_spec]
impl u64 {
    #[sig(fn(num: u64) -> u32{r: (num == 0 => r == 64) &&
                                 (num > 0 => r <= 63) &&
                                 (num > 1 => r <= 62)
                            })]
    fn leading_zeros(self) -> u32;
}

// Only works when usize is 32-bits
#[flux_rs::extern_spec]
impl usize {
    #[sig(fn(num: usize{num <= u32::MAX}) -> u32{r: (num == 0 => r == 32) && (num > 0 => r <= 31) && (num >= 512 => r <= 22) && (num < 512 => r > 22)})]
    fn leading_zeros(self) -> u32;

    #[sig(fn(num: usize{num <= u32::MAX}) -> u32{r: (num == 0 => r == 32) && (num > 0 => r <= 31) && (num == u32::MAX => r == 0)})]
    fn trailing_zeros(self) -> u32;

    #[sig(fn (num: usize{num <= u32::MAX}) -> u32{r: r <= 32})]
    fn count_ones(self) -> u32;

    #[sig(fn(num: usize, rhs: usize) -> usize[if num < rhs { 0 } else { num - rhs }])]
    fn saturating_sub(self, rhs: usize) -> usize;

    #[sig(fn(num: usize, rhs: usize) -> usize[(num + rhs - 1) / rhs] requires rhs > 0)]
    fn div_ceil(self, rhs: usize) -> usize;
}
