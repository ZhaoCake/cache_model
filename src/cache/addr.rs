#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressParts {
    pub tag: u32,
    pub index: usize,
    pub offset: usize,
}

pub fn decode_address(addr: u32, line_size: usize, num_lines: usize) -> AddressParts {
    // TODO: 实现地址拆分：TAG + Index + Offset
    let _ = (addr, line_size, num_lines);
    todo!("decode_address not implemented");
}
