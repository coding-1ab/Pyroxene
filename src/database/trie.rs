//0<Nibble<0f
pub type Nibble = u8;

pub enum Node {
    Extension{
        nibble: Vec<Nibble>,
        child: Node
    }
    Branch{
        children: [Node;16],

    }
}