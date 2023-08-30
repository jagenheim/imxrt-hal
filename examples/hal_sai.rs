//! A quick test of the SAI peripheral

#![no_std]
#![no_main]

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        _,
        board::Specifics {
            mut sai, // SAI peripheral
            ..
        },
    ) = board::new();

    let mut a: u16 = 0;
    let mut b: u16 = 100;
    loop {
        //sai.send(a).unwrap();

        if a >= (65535 - b) {
            a = 0;
            b += 1;
            if b > 400 {
                b = 100;
            }
        }
        a += b;
    }
}
