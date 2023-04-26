struct AThing;

pub mod alpha {
    use delta::ATrait;
    pub mod beta {
        use crate::AThing;
        use chrono::DateTime;

        pub struct AnotherThing {
            dt: chrono::Duration,
        }

        pub mod gamma {

            use rand::CryptoRng;
        }
    }
    pub mod delta {
        use super::beta::AnotherThing;
        use rand::Rng;

        pub trait ATrait {}
    }
}
