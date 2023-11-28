trait HashData {
    fn as_64s<const SIZE: usize>(&mut self) -> &mut [u64; SIZE];
    fn as_32s<const SIZE: usize>(&mut self) -> &mut [u32; SIZE];
}

struct Hash256([u8; 32]);
impl HashData for Hash256 {
    fn as_64s<const SIZE: usize>(&mut self) -> &mut [u64; SIZE] {
        unsafe {
            return std::mem::transmute::<&mut [u8; 32], &mut [u64; SIZE]>(&mut self.0);
        }
    }

    fn as_32s<const SIZE: usize>(&mut self) -> &mut [u32; SIZE] {
        unsafe {
            return std::mem::transmute::<&mut [u8; 32], &mut [u32; SIZE]>(&mut self.0);
        }
    }
}
fn main() {
    unsafe {
        let mut x = Hash256([0u8; 32]);

        println!("{:?}", x.0);

        x.as_64s()[1] += 1_u64.to_be();
        x.as_32s()[1] += 1;
        x.0[1] += 1;

        println!("{:?}", x.0);
    }
}
