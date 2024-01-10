use std::{slice::{Iter, IterMut}, vec::IntoIter};

#[derive(Clone, Debug)]
pub struct VecTree<V> {
    pub v: V,
    childs: Vec<VecTree<V>>,
}

impl<V> VecTree<V>
{
    pub fn new(v: V) -> Self {
        VecTree {
            v,
            childs: vec![]
        }
    }
    
    pub fn node(&self, i: usize) -> Option<&VecTree<V>> {
        self.childs.get(i)
    }

    pub fn node_mut(&mut self, i: usize) -> Option<&mut VecTree<V>> {
        self.childs.get_mut(i)
    }

    pub fn nodes_iter(&self) -> Iter<'_, VecTree<V>> {
        self.childs.iter()
    }

    pub fn nodes_iter_mut(&mut self) -> IterMut<'_, VecTree<V>> {
        self.childs.iter_mut()
    }

    pub fn nodes_into_iter(self) -> IntoIter<VecTree<V>> {
        self.childs.into_iter()
    }

    pub fn push(&mut self, v: V)  {
        self.childs.push(VecTree::new(v))
    }

    pub fn count_children(&self) -> usize {
        self.childs.len()
    }
}
