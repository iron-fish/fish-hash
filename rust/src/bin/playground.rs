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

        x.as_64s()[1] += 1_u64.to_le();
        x.as_32s()[1] += 1;
        x.0[1] += 1;

        println!("{:?}", x.0);

        println!("cast as 64s: {:?}", x.as_64s());

        let x_ptr = x.0.as_mut_ptr();
        // let y = x_ptr as *mut u64;
        // println!("x_ptr: {:?}\ny {:?}", x_ptr, &*y);
        // println!("y {:?}", &*y.offset(1));

        let y_ptr: *mut u64 = x_ptr.cast();
        println!("{:?} {:?}", x_ptr.read(), x_ptr.offset(1).read());
        println!(
            "{:?} {:?} {:?} {:?} {:?}",
            y_ptr.read(),
            y_ptr.offset(1).read(),
            y_ptr.offset(2).read(),
            y_ptr.offset(3).read(),
            y_ptr.offset(4).read(), // OOB read
        );

        let mut y_ptr: *mut u64 = x_ptr.cast();
        for _ in 0..x.0.len() / 8 {
            println!("y value: {:?}", y_ptr.read());
            y_ptr = y_ptr.offset(1);
        }
    }
}
