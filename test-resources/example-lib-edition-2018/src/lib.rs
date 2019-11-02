mod A {
    pub struct B {}
}

pub mod submodule;

use crate::A as OtherA;
use ::chrono;
use std::fmt::Write;
use A::{self as OtherA2, B};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
