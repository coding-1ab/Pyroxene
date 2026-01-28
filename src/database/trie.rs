//0<Nibble<0f
pub type Nibble = u8;

fn byte_to_nibble(bytes:&[u8])->Vec<Nibble>{
    let ret:Vec<Nibble> = vec![];
    for i in bytes{
        
    }
}

pub enum Node {
    Extension{
        nibble: Vec<Nibble>,
        child: Node
    },
    Branch{
        children: [Node;16],
        value: u128
    },
    Leaf{
        key: [Nibble;64],
        value: u128
    }
}

