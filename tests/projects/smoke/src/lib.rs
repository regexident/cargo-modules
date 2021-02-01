mod orphans;

mod uses {
    use core::ops::{self, Add, Eq};
    use std::fmt::{self, Debug};

    use package_lib_target::dummy;

    use crate::hierarchy;

    mod cycle {
        mod node_0 {
            use super::cycle_1;
        }

        mod node_1 {
            mod node_2 {
                use super::super::cycle_0;
            }
        }
    }
}

mod hierarchy {
    mod lorem {
        mod ipsum {}
        mod dolor {
            mod sit {
                mod amet {}
            }
        }
        mod consectetur {
            mod adipiscing {
                mod elit {}
            }
        }
    }
}

mod visibility {
    mod mods {
        pub mod pub_public {}
        pub(crate) mod pub_crate {}
        pub(in crate::lorem) mod pub_module {}
        pub(super) mod pub_super {}
        mod pub_private {}
    }

    mod structs {
        pub struct PubPublic {}
        pub(crate) struct PubCrate {}
        pub(in crate::lorem) struct PubModule {}
        pub(super) struct PubSuper {}
        struct PubPrivate {}
    }

    mod enums {
        pub enum PubPublic {}
        pub(crate) enum PubCrate {}
        pub(in crate::lorem) enum PubModule {}
        pub(super) enum PubSuper {}
        enum PubPrivate {}
    }

    mod unions {
        pub union PubPublic {}
        pub(crate) union PubCrate {}
        pub(in crate::lorem) union PubModule {}
        pub(super) union PubSuper {}
        union PubPrivate {}
    }

    mod fns {
        pub fn pub_public() {}
        pub(crate) fn pub_crate() {}
        pub(in crate::lorem) fn pub_module() {}
        pub(super) fn pub_super() {}
        fn pub_private() {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
