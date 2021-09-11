pub mod a {
    use self::b::X;
    use self::d::Y;

    pub mod b {
        use self::c;
        pub use c::X;

        mod c {
            pub struct X {}
        }
    }

    pub mod d {
        use self::e;
        pub type Y = e::Y;

        mod e {
            pub struct Y {}
        }
    }
}
