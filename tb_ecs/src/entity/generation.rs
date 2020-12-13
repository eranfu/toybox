#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct Generation {
    gen: i32,
}

impl Generation {
    pub(crate) fn new_alive() -> Self {
        Self { gen: 0 }
    }

    pub(crate) fn die(&mut self) {
        assert!(self.is_alive());
        self.gen = -self.gen - 1;
    }

    pub(crate) fn relive(&mut self) {
        assert!(!self.is_alive());
        self.gen = self
            .gen
            .checked_neg()
            .expect("generation checked_neg failed");
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.gen >= 0
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::generation::Generation;

    #[test]
    #[should_panic(expected = "generation checked_neg failed")]
    fn validate_gen() {
        let mut gen = Generation::new_alive();
        assert!(gen.is_alive());
        gen.gen = i32::max_value();
        gen.die();
        assert!(!gen.is_alive());
        gen.relive();
        assert!(!gen.is_alive()); // 因为溢出了，所以不会变为有效
    }
}
