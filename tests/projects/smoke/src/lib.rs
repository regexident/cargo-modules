mod orphans;

mod uses {
    use core::{cmp, ops};
    use std::fmt;

    use crate::hierarchy;

    mod cycle {
        mod node_0 {
            use super::node_1;
        }

        mod node_1 {
            mod node_2 {
                use super::super::node_0;
            }
        }
    }
}

mod hierarchy {
    mod lorem {
        struct Lorem;

        mod ipsum {
            struct Ipsum;
        }
        mod dolor {
            struct Dolor;

            mod sit {
                struct Sit;

                mod amet {
                    struct Amet;
                }
            }
        }
        mod consectetur {
            struct Consectetur;

            mod adipiscing {
                struct Adipiscing;

                mod elit {
                    struct Elit;
                }
            }
        }
    }
}

mod visibility {
    mod dummy {
        mod mods {
            pub mod pub_public {}
            pub(crate) mod pub_crate {}
            pub(in crate::visibility) mod pub_module {}
            pub(super) mod pub_super {}
            mod pub_private {}
        }

        mod structs {
            pub struct PubPublic {}
            pub(crate) struct PubCrate {}
            pub(in crate::visibility) struct PubModule {}
            pub(super) struct PubSuper {}
            struct PubPrivate {}
        }

        mod enums {
            pub enum PubPublic {}
            pub(crate) enum PubCrate {}
            pub(in crate::visibility) enum PubModule {}
            pub(super) enum PubSuper {}
            enum PubPrivate {}
        }

        mod unions {
            pub union PubPublic {
                dummy: (),
            }
            pub(crate) union PubCrate {
                dummy: (),
            }
            pub(in crate::visibility) union PubModule {
                dummy: (),
            }
            pub(super) union PubSuper {
                dummy: (),
            }
            union PubPrivate {
                dummy: (),
            }
        }

        mod traits {
            mod safe {
                pub trait PubPublic {}
                pub(crate) trait PubCrate {}
                pub(in crate::visibility) trait PubModule {}
                pub(super) trait PubSuper {}
                trait PubPrivate {}
            }

            mod r#unsafe {
                pub unsafe trait PubPublic {}
                pub(crate) unsafe trait PubCrate {}
                pub(in crate::visibility) unsafe trait PubModule {}
                pub(super) unsafe trait PubSuper {}
                unsafe trait PubPrivate {}
            }
        }

        mod fns {
            pub fn pub_public() {}
            pub(crate) fn pub_crate() {}
            pub(in crate::visibility) fn pub_module() {}
            pub(super) fn pub_super() {}
            fn pub_private() {}
        }

        mod kinds {
            mod Module {}

            struct Struct {}
            enum Enum {}
            union Union {
                dummy: (),
            }

            trait Trait {}
            unsafe trait UnsafeTrait {}

            type TraitAlias = Trait;
            type TypeAlias = Struct;

            fn Function() {}
            const fn ConstFunction() {}
            async fn AsyncFunction() {}
            unsafe fn UnsafeFunction() {}

            const const_binding: bool = false;
            static static_binding: bool = false;

            macro_rules! Macro {
                () => {};
            }
        }
    }
}

mod functions {
    struct Local;
    type Core = i32;
    type Std = String;

    fn inputs(local: Local, core: Core, std: Std) {
        unimplemented!()
    }

    fn outputs() -> (Local, Core, Std) {
        unimplemented!()
    }

    fn body() {
        let local: Local;
        let core: Core = 42;
        let std: Std = "hello world".to_owned();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
