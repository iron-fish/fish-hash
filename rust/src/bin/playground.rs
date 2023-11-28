struct Hash256([u8; 32]);

impl Hash256 {
    unsafe fn as_64s(&mut self) -> &mut [u64; 4] {
        std::mem::transmute(&mut self.0)
    }

    unsafe fn as_32s(&mut self) -> &mut [u32; 8] {
        std::mem::transmute(&mut self.0)
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

        println!("cast as 64s: {:?}", x.as_64s());

        let mut x_ptr = x.0.as_ptr();
        let mut y = x_ptr as *mut u64;
        println!("x_ptr: {:?}\ny {:?}", x_ptr, &*y);
        y += (std::mem::size_of::<u64>()) as u64;
        println!("y {:?}", &*y);
    }
}
