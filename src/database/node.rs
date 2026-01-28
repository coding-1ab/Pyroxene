use std::sync::Arc;
use crate::database::nibble::Nibble;

pub enum Node {
    Empty,
    Extension(Arc<ExtensionNode>),
    Branch(Arc<BranchNode>),
    Leaf(Arc<LeafNode>)
}

impl Node {
    pub fn from_leaf(key:[Nibble;64], value: u128) ->Self {
        let leaf = Arc::new(LeafNode{key:key, value:value});
        Node::Leaf(leaf)
    }
    pub fn from_branch(children: [Node; 16], value: Option<u128>) -> Self{
        let branch = Arc::new(BranchNode{children:children,value:value});
        Node::Branch(branch)
    }
    pub fn from_extendsion(nibbles:&[Nibble], child: Node) -> Self{
        let extension = Arc::new(ExtensionNode{nibbles:nibbles.to_vec(), child});
        Node::Extension(extension)
    }
}

pub struct BranchNode{
    children: [Node;16],
    value: Option<u128>
}

pub struct LeafNode{
    key: [Nibble;64],
    value: u128
}

pub struct ExtensionNode{
    nibbles: Vec<Nibble>,
    child: Node
}

impl BranchNode{
    pub fn insert(&mut self, i: usize, n: Node){
        if i==16 {
            match n{
                Node::Leaf(leaf) => {
                    self.value = Some(leaf.value.clone());
                },
                _ => panic!("type of n is must be leaf")
            }
        }else{
            self.children[i] = n
        }
    }
}

pub fn empty_children() -> [Node; 16] {
    [
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
        Node::Empty,
    ]
}