pub mod foobar;

mod phoo;

// TODO: Fix this
use self::phoo::Phoo;

#[derive(Default)]
pub struct Foo {
    #[allow(dead_code)]
    phoo: Phoo,
}
