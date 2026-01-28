//u8 but 
pub type Nibble = u8;

pub enum Node {
    Extension{
        nibble: Vec<Nibble>,
        child: Node
    }
    
}