use dam_application::Randomness;

pub struct OsRandom;

impl Randomness for OsRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        // The OS source failing is not recoverable for an identifier; a zeroed
        // id would collide, so this is the one place a panic is correct.
        getrandom::fill(buf).expect("the operating system random source is unavailable");
    }
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
