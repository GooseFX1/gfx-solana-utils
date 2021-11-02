use num_traits::AsPrimitive;

pub trait ApplyDecimal {
    fn apply<N: AsPrimitive<f64>>(self, amount: N) -> u64;
    fn unapply<N: AsPrimitive<u64>>(self, amount: N) -> f64;
}

impl ApplyDecimal for u8 {
    fn apply<N: AsPrimitive<f64>>(self, amount: N) -> u64 {
        (amount.as_() * 10f64.powf(self as f64)) as u64
    }
    fn unapply<N: AsPrimitive<u64>>(self, amount: N) -> f64 {
        amount.as_() as f64 / 10u64.pow(self as u32) as f64
    }
}
