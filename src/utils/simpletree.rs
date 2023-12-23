#[derive(Debug, Clone)]
pub struct SimpleTree<T> {
    pub elem: T,
    childs: Vec<SimpleTree<T>>
}

impl<T> SimpleTree<T> {
    pub fn new(elem: T) -> Self {
        SimpleTree { elem, childs: vec![] }
    }

    pub fn get_children(&self) -> &Vec<SimpleTree<T>> {
        &self.childs
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<SimpleTree<T>> {
        &mut self.childs
    }

    pub fn add_child(&mut self, child: T) -> &mut SimpleTree<T> {
        self.childs.push(SimpleTree { elem: child, childs: vec![] });

        self.childs.last_mut().unwrap()
    }
}