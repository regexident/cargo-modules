mod bar;
mod foo;

use crate::foo::Foo;

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
