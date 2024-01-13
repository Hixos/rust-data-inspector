use std::{slice::{Iter, IterMut}, vec::IntoIter};

#[derive(Clone, Debug)]
pub struct VecTree<V> {
    pub value: V,
    pub children: Vec<VecTree<V>>,
}

impl<V> VecTree<V>
{
    pub fn new(v: V) -> Self {
        VecTree {
            value: v,
            children: vec![]
        }
    }
}
