//! Synchronous Audio Interface
//! FIXME:
//! Would need some example code here too
//! and documentation of course

use crate::ral;

/// FIXME: Some documentation would be nice
pub struct Sai<const N: u8> {
    sai: ral::sai::Instance<N>,
}

/// SAI1 alias
pub type Sai1 = Sai<1>;
// FIXME: Add more aliases

impl<const N: u8> Sai<N> {
    pub fn new(sai: ral::sai::Instance<N>) -> Self {
        // Reset the all SAI registers
        //ral::modify_reg!(ral::sai, sai, CR, SWR: 1);
        // Wait for reset can be done like this:
        //while ral::read_reg!(ral::sai, sai, CR, SWR == 1) {}

        Self { sai }
    }

    // What can you do with a SAI?
    pub fn read_version(sau: ral::sai::Instance<N>) -> (u32, u32) {
        let major = ral::read_reg!(ral::sai, sai, VERID, MAJOR);
        let minor = ral::read_reg!(ral::sai, sai, VERID, MINOR);
        (major, minor)
    }
}
