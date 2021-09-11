use crate::imported::Placebo;

#[cfg(test)]
use crate::imported::OnlyUsedWithTest;

#[cfg(not(test))]
use crate::imported::OnlyUsedWithoutTest;

pub struct Placebo {}

#[cfg(test)]
pub struct OnlyExistsWithTest {}

#[cfg(not(test))]
pub struct OnlyExistsWithoutTest {}

pub mod imported {
    pub struct Placebo {}

    pub struct OnlyUsedWithTest {}

    pub struct OnlyUsedWithoutTest {}
}

pub mod importing {
    use crate::imported::Placebo;

    #[cfg(test)]
    use crate::imported::OnlyUsedWithTest;

    #[cfg(not(test))]
    use crate::imported::OnlyUsedWithoutTest;
}

#[cfg(test)]
pub mod only_exists_with_test {
    pub struct Placebo {}

    #[cfg(test)]
    pub struct OnlyExistsWithTest {}

    #[cfg(not(test))]
    pub struct OnlyExistsWithoutTest {}
}

// Somehow this module and its contents end up getting omitted:
#[cfg(not(test))]
pub mod only_exists_without_test {
    pub struct Placebo {}

    #[cfg(test)]
    pub struct OnlyExistsWithTest {}

    #[cfg(not(test))]
    pub struct OnlyExistsWithoutTest {}
}
