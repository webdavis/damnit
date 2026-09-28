use dam_application::Randomness;

pub struct OsRandom;

impl Randomness for OsRandom {
    fn fill(&self, buf: &mut [u8]) {
        fill_or_panic_since_a_zeroed_id_would_collide(buf);
    }
}

fn fill_or_panic_since_a_zeroed_id_would_collide(buf: &mut [u8]) {
    getrandom::fill(buf).expect("the operating system random source is unavailable");
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::Randomness;

    #[test]
    fn two_fills_differ() {
        let mut a = [0u8; 16];
        let mut b = [0u8; 16];
        OsRandom.fill(&mut a);
        OsRandom.fill(&mut b);
        assert_ne!(a, b);
    }
}
