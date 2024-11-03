struct Skeleton<'a, T>
where
    T: Uniplate,
{
    clean: bool,                // clean/dirty flag
    node: &'a T,                // reference to existing node in tree
    children: Vec<Skeleton<T>>, // skeletons which contain references to this skeleton's node's children.
}

impl<'a> Skeleton<'a, T>
where
    T: Uniplate,
{
    pub fn new(node: &'a T) -> Skeleton<'a, T> {
        Skeleton {
            clean: false,
            node,
            children: node
                .children()
                .iter()
                .map(|child| Skeleton::new(child))
                .collect(),
        }
    }

    pub fn node(&self) -> &T {
        self.node
    }

    pub fn mark_clean(&mut self) {
        self.clean = true;
    }

    pub fn is_clean(&self) -> bool {
        self.clean
    }
}
