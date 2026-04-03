#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressParts {
    pub tag: u32,
    pub index: usize,
    pub offset: usize,
}

pub fn decode_address(addr: u32, line_size: usize, num_lines: usize) -> AddressParts {
    AddressParts {
        tag: addr / (line_size as u32 * num_lines as u32), 
        // tag 是地址除以整个 cache 大小（line_size * num_lines），得到高位部分
        index: ((addr / line_size as u32) % num_lines as u32) as usize,
        // index 是地址除以 line_size 后对 num_lines 取模，得到中间部分
        offset: (addr % line_size as u32) as usize,
        // offset 是地址对 line_size 取模，得到低位部分
    }
}
