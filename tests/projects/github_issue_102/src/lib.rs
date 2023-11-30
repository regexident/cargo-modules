pub mod a {
    use self::b::X;

    pub mod b {
        pub struct X {}
    }
    pub mod c {
        pub struct Y {}
    }

    pub struct Z {
        x: X,
        y: c::Y,
    }
}
