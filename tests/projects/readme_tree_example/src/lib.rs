pub trait Lorem {
    fn ipsum();
}

mod dolor {
    pub(crate) enum Sit {
        Bool(bool),
    }
}

mod amet {
    mod consectetur {
        mod adipiscing {
            pub(in crate::amet) union Elit {
                a: bool,
                b: usize,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
