//0<Nibble<0f
pub type Nibble = u8;

fn byte_to_nibble(bytes:&[u8])->Vec<Nibble>{
    let ret:Vec<Nibble> = vec![];
    for _i in bytes{

    }
    ret
}

pub enum Node {
    Extension{
        nibble: Vec<Nibble>,
        child: Box<Node>
    },
    Branch{
        children: [Box<Node>;16],
        value: u128
    },
    Leaf{
        key: [Nibble;64],
        value: u128
    }
}

