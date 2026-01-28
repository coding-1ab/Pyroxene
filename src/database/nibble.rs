pub type Nibble = u8;

fn byte_to_nibble(bytes:&[u8])->Vec<Nibble>{
    let mut ret:Vec<Nibble> = vec![];
    for i in bytes{
        let high_nibble = (i >> 4) & 0x0f;
        let low_nibble = i & 0x0f;
        ret.push(high_nibble);
        ret.push(low_nibble);
    }
    ret
}