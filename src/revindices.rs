#[macro_export]
macro_rules! revinds{
    () => {
        {
            const fn generate() -> [u32; 100000] {
                let mut arr = [0u32; 100000];
    
                let mut x = 100000u32;
                while x > 0{
                    x -= 1;
                    arr[x as usize] = x;
                }
                arr
            }
            generate()
        }
    };
}





pub const REV_INDS: [u32; 100000] = revinds!();