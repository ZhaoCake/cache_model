#[derive(Debug, Clone)]
pub struct CacheLine {
    pub valid: bool,
    pub dirty: bool,
    pub tag: u32,
    pub data: LineData,
}

pub type LineData = Vec<u8>;

impl CacheLine {
    pub fn new(line_size: usize) -> Self {
        Self {
            valid: false,
            dirty: false,
            tag: 0,
            data: vec![0; line_size],
            // vec![0; line_size] 会创建一个长度为 line_size 的 Vec<u8>，并用 0 初始化每个元素
        }
    }

    pub fn is_hit(&self, tag: u32) -> bool {
        self.valid && self.tag == tag
    }

    // 什么情况会调用read u32，要考虑清楚
    pub fn read_u32(&self, offset: usize, rsize: u8) -> u32 {
        // offset只能是0/1/2/3，rsize只能是1/2/4
        match rsize {
            1 => self.data[offset] as u32,
            2 => {
                if offset % 2 != 0 {
                    panic!("Unaligned read");
                }
                let bytes = &self.data[offset..offset + 2];
                u16::from_le_bytes(bytes.try_into().unwrap()) as u32
                // from_le_bytes 是小端字节序，try_into().unwrap() 将 &[u8] 转换为 [u8; 2]
            }
            4 => {
                if offset % 4 != 0 {
                    // 实际上也是offset == 0 因为line_size是4B。但是为了之后扩展还是用%4
                    panic!("Unaligned read");
                }
                let bytes = &self.data[offset..offset + 4];
                u32::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => panic!("Invalid read size"),
        }
    }

    pub fn write_u32(&mut self, offset: usize, wdata: u32, wmask: u8) {
        // wmask 的每一位表示对应字节是否要写入
        for i in 0..4 {
            if (wmask & (1 << i)) != 0 {
                self.data[offset + i] = (wdata >> (i * 8)) as u8;
            }
        }
        self.dirty = true;
    }

    pub fn fill(&mut self, tag: u32, data: &[u8]) {
        // data是u8数组，长度应该等于line_size，调用者保证这一点
        // TODO: 填充整行数据
        self.valid = true;
        self.dirty = false;
        self.tag = tag;
        self.data.copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_line_new() {
        let line = CacheLine::new(4);
        assert!(!line.valid);
        assert!(!line.dirty);
        assert_eq!(line.tag, 0);
        assert_eq!(line.data, vec![0; 4]);
    }

    #[test]
    fn test_cache_line_is_hit() {
        let mut line = CacheLine::new(4);
        line.valid = true;
        line.tag = 1;
        assert!(line.is_hit(1));
        assert!(!line.is_hit(2));
    }

    #[test]
    fn test_cache_line_read_u32() {
        let mut line = CacheLine::new(4);
        line.valid = true;
        line.tag = 1;
        line.data = vec![1, 2, 3, 4];
        assert_eq!(line.read_u32(0, 4), 0x04030201);
    }

    #[test]
    fn test_cache_line_write_u32() {
        let mut line = CacheLine::new(4);
        line.valid = true;
        line.tag = 1;
        line.write_u32(0, 0x04030201, 0xF);
        assert_eq!(line.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_cache_line_fill() {
        let mut line = CacheLine::new(4);
        line.fill(1, &[1, 2, 3, 4][..]); // &[1, 2, 3, 4][..] 是将数组转换为切片,否则&[u8;4]类型不匹配
        assert!(line.valid);
        assert!(!line.dirty);
        assert_eq!(line.tag, 1);
        assert_eq!(line.data, vec![1, 2, 3, 4]);
    }
}
