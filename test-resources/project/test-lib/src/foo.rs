pub mod foobar;

mod phoo;

use self::phoo::Phoo;

#[derive(Default)]
pub struct Foo {
    #[allow(dead_code)]
    phoo: Phoo,
}
