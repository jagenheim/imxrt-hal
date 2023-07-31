//! A quick test of the SAI peripheral
//! Need to extend the documentation here at some point

#![no_std]
#![no_main]

use imxrt_hal::sai;
use imxrt_ral::sai::SAI1;

#[imxrt_rt::entry]
fn main() -> ! {
    let (
        _,
        board::Specifics {
            sai, // SAI peripheral
            ..
        },
    ) = board::new();

    let mut sai = sai::Sai::new(unsafe { SAI1::instance() });
    // sai.set_XXX

    let mut a: u16 = 0;
    let mut b: u16 = 100;
    loop {
        //sai.send(s).unwrap();
        //sai.send(s).unwrap();

        if s >= (65535 - y) {
            s = 0;
            y += 1;
            if y > 400 {
                y = 100;
            }
        }
        s += y;
    }
}
