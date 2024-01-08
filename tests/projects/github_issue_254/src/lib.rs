trait Dummy {
    fn dummy(&self);
}

impl<T> Dummy for T
where
    T: Clone,
{
    fn dummy(&self) {}
}
