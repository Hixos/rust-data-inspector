use std::{
    collections::{
        hash_map::{IntoIter, Iter, IterMut},
        HashMap,
    },
    hash::Hash,
};

#[derive(Clone, Debug)]
pub struct HashTree<K, V> {
    pub v: V,
    childs: HashMap<K, HashTree<K, V>>,
}

#[allow(dead_code)]
impl<K, V> HashTree<K, V>
where
    K: Hash + Eq,
{
    pub fn new(v: V) -> Self {
        HashTree {
            v,
            childs: HashMap::default(),
        }
    }
    
    pub fn node(&self, k: &K) -> Option<&HashTree<K, V>> {
        self.childs.get(k)
    }

    #[allow(dead_code)]
    pub fn contains_key(&self, k: &K) -> bool {
    #[allow(dead_code)]
        self.childs.contains_key(k)
    }

    pub fn node_mut(&mut self, k: &K) -> Option<&mut HashTree<K, V>> {
        self.childs.get_mut(k)
    }

    pub fn nodes_iter(&self) -> Iter<'_, K, HashTree<K, V>> {
        self.childs.iter()
    }

    pub fn nodes_iter_mut(&mut self) -> IterMut<'_, K, HashTree<K, V>> {
        self.childs.iter_mut()
    }

    pub fn nodes_into_iter(self) -> IntoIter<K, HashTree<K, V>> {
        self.childs.into_iter()
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.childs
            .insert(
                k,
                HashTree {
                    v,
                    childs: HashMap::new(),
                },
            )
            .map(|v| v.v)
    }

    pub fn count_children(&self) -> usize {
        self.childs.len()
    }
}
