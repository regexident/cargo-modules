mod bar;
mod foo;

// TODO: Fix this
use crate::foo::Foo;

// TODO: Fix this
pub use crate::bar::Bar;

pub fn make_foo() {
    Foo::default();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
