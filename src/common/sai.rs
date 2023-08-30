//! Synchronous Audio Interface
//! FIXME:
//! Would need some example code here too
//! and documentation of course

use crate::{ccm, ral, iomuxc};

/// FIXME: This should be able to use a configurator for I2S/MQS/SPDIF
/// Currently, we build it around I2S, because why not?


/// I2S pins
///
/// The audio chip, part number SGTL5000, connects to Teensy using 7 signals. The I2C pins SDA and
/// SCL are used to control the chip and adjust parameters. Audio data uses I2S signals, DIN (which
/// the audio shield uses to make analog signals at the headphones and line out) and DOUT (which
/// the audio shield creates from reading the analog signals at line in or microphone), and 3
/// clocks, LRCLK (44.1 kHz), BCLK (1.41 or 2.82 MHz) and MCLK (11.29 MHz). All 3 clocks are
/// created by Teensy. Most I2S software on Teensy uses 64 bits per LRCLK (BCLK=2.82 MHz), where
/// the upper 16 bit of each 32 bit word are used and the lower 16 bits are ignored. However, some
/// I2S support uses only 32 bits per LRCLK (BCLK=1.41 MHz). Refer to the Design Tool documentation
/// for each I2S input or output for details. The SGTL5000 listens for clocks. All its clock pins
/// are inputs. 
///

pub struct Sai<PINS, const N: u8> {
    sai: ral::sai::Instance<N>,
    pins: PINS,
}

pub struct Pins<MCLK, BCLK, LRCLK, DATA>
{
    /// Master clock
    pub mclk: MCLK,
    /// Bit clock
    pub bclk: BCLK,
    /// Word clock
    pub lrclk: LRCLK,
    /// Data pin
    pub data: DATA,
}

/// SAI1 alias
pub type Sai1<MCLK, BCLK, LRCLK, DATA> = Sai<Pins<MCLK, BCLK, LRCLK, DATA>, 1>;
// FIXME: Add more aliases

const AUDIO_SAMPLE_RATE: f64 = 44100.0;

impl<MCLK, BCLK, LRCLK, DATA, const N: u8> Sai<Pins<MCLK, BCLK, LRCLK, DATA>, N> 
where
    MCLK: iomuxc::sai::Pin<iomuxc::consts::Const<N> >,
    BCLK: iomuxc::sai::Pin<iomuxc::consts::Const<N> >,
    LRCLK: iomuxc::sai::Pin<iomuxc::consts::Const<N> >,
    DATA: iomuxc::sai::Pin<iomuxc::consts::Const<N> >,
{
    /// Create a new SAI
    pub fn new(
        sai: ral::sai::Instance<N>,
        mut pins: Pins<MCLK, BCLK, LRCLK, DATA>,
    ) -> Self {
        iomuxc::sai::prepare(&mut pins.mclk);
        iomuxc::sai::prepare(&mut pins.bclk);
        iomuxc::sai::prepare(&mut pins.lrclk);
        iomuxc::sai::prepare(&mut pins.data);
        Self::init(sai, pins)
    }
}

impl<PINS, const N: u8> Sai<PINS, N> {
    /// The peripheral instance.
    pub const N: u8 = N;

    fn init(mut sai : ral::sai::Instance<N>,
            pins: PINS,
        ) -> Self {
        let mut ccm = unsafe { ral::ccm::CCM::instance() };
        let mut ccm_analog = unsafe { ral::ccm_analog::CCM_ANALOG::instance() };
        let mut iomuxc = unsafe { ral::iomuxc::IOMUXC::instance() };
        let mut iomuxc_gpr = unsafe { ral::iomuxc_gpr::IOMUXC_GPR::instance() };
        // Reset the all SAI registers
        //ral::modify_reg!(ral::sai, sai, CR, SWR: 1);
        // Wait for reset can be done like this:
        //while ral::read_reg!(ral::sai, sai, CR, SWR == 1) {}

        // Setup CCM Clock Gating Register. CG9 enables the SAI1 clock
        // ral::modify_reg!(ra1::ccm, ccm, CCGR5, CG9: 1);
        ccm::clock_gate::sai::<N>().set(&mut ccm, ccm::clock_gate::ON);

        // Setup Analog PLL for SAI1_CLK_ROOT, not sure how to do this generically
        // over SAI<N>, matching over N?
        //
        // For audio, we should probably look at PLL4 (786.43).
        // This is divided by /1 overriden w/ PLL_AUDIOn[POST_DIV_SELECT]
        // Then divided by /1 overriden w/ CCM_ANALOG_MISC2n[MSB:LSB]
        // Divided by /4 CS1DR[SAIn_CLK_PRED]
        // Then Divided by /2 CS1DR[SAIn_CLK_PODF]
        // This results in SAI1_CLK_ROOT
        // Page 1010 in the rev3 manual

        // FIXME: Implement imxrt_hal::ccm::analog::pll4

        // PLL4 can be between 648MHz or 1300Mhz. The reference clock is 24Mhz
        // PLL output frequency = Fref * ( DIV_SELECT + NUM / DENOM )

        // This was copied verbatim from the Teensy library.
        // This should result in a clock for SAI that is 256 times faster than the AUDIO_SAMPLE_RATE

        let n1 = 4;
        let n2: u32 = 1 + (24_000_000 * 27) / (AUDIO_SAMPLE_RATE as u32 * 256 * n1);
        let tmp: f64 = (AUDIO_SAMPLE_RATE * 256f64 * (n1 * n2) as f64) / 24_000_000f64;
        let c0: u32 = tmp as u32;
        let c2 = 10_000;
        let c1 = (tmp * c2 as f64 - (c0 * c2) as f64) as u32;

        // divider can be 27-54
        ral::write_reg!(
            ral::ccm_analog,
            ccm_analog,
            PLL_AUDIO,
            BYPASS : 1,
            ENABLE : 1,
            POST_DIV_SELECT : POST_DIV_SELECT_2,
            DIV_SELECT : c0
        );

        // numerator of Factional loop divider
        ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_NUM, A : c1);

        // denominator of Fractional loop divider
        ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_DENOM, B : c2);

        // Turn on PLL and wait for PLL lock
        ral::modify_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, POWERDOWN : 0);
        while ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, LOCK == 1) {}

        // Disable bypass after setting up PLL
        ral::modify_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, BYPASS : 0);

        // Setup the SAI registers
        // Clock setup, for PLL4 and using n1 and n2 as divider. Need to subtract by one since they
        // start counting from 0
        ral::modify_reg!(ral::ccm, ccm, CSCMR1, SAI1_CLK_SEL : SAI1_CLK_SEL_2);
        ral::modify_reg!(
            ral::ccm,
            ccm,
            CS1CDR,
            SAI1_CLK_PRED: n1 - 1,
            SAI1_CLK_PODF: n2 - 1
        );

        // Select MCLK and set it up as an output signal
        ral::modify_reg!(
            ral::iomuxc_gpr,
            iomuxc_gpr,
            GPR1,
            SAI1_MCLK_DIR : SAI1_MCLK_DIR_1
        );
        ral::modify_reg!(
            ral::iomuxc_gpr,
            iomuxc_gpr,
            GPR1,
            SAI1_MCLK1_SEL : SAI1_MCLK1_SEL_0
        );

        // Setting up core pins
        // PIN23 - MCLK
        ral::write_reg!(
            ral::iomuxc,
            iomuxc,
            SW_MUX_CTL_PAD_GPIO_AD_B1_09,
            MUX_MODE : ALT3
        ); // SAI1_MCLK
           // PIN21 - BCLK to audio board
        ral::write_reg!(
            ral::iomuxc,
            iomuxc,
            SW_MUX_CTL_PAD_GPIO_AD_B1_11,
            MUX_MODE : ALT3
        ); // SAI1_RX_BCLK
           // PIN20 - LRCLK to audio board
        ral::write_reg!(
            ral::iomuxc,
            iomuxc,
            SW_MUX_CTL_PAD_GPIO_AD_B1_10,
            MUX_MODE : ALT3
        ); // SAI1_RX_SYNC

        // Actually set up the SAI
        // Transmit word mask enabled
        ral::write_reg!(ral::sai, sai, TMR, TWM : 0);
        // transmit Watermark
        ral::write_reg!(ral::sai, sai, TCR1, TFW : 1);
        // transmit settings
        ral::write_reg!(
            ral::sai,
            sai,
            TCR2,
            DIV : 1,
            BCD : 1,
            BCP : 1,
            MSEL : MSEL_1,
            SYNC : SYNC_1
        );
        // transmit settings
        ral::write_reg!(ral::sai, sai, TCR3, TCE : 1);
        ral::write_reg!(
            ral::sai,
            sai,
            TCR4,
            FSD : FSD_1,
            FSP : FSP_1,
            FSE : FSE_1,
            MF : MF_1,
            SYWD : 32 - 1,
            FRSZ : 2 - 1
        );
        ral::write_reg!(ral::sai, sai, TCR5, WNW : 31, W0W : 31, FBT : 31);

        // Receive settings
        ral::write_reg!(ral::sai, sai, RMR, RWM : RWM_0);
        ral::write_reg!(ral::sai, sai, RCR1, RFW : 1);
        ral::write_reg!(
            ral::sai,
            sai,
            RCR2,
            BCP : BCP_1,
            SYNC : SYNC_0,
            BCD : BCD_1,
            DIV : 1, // (DIV + 1) * 2
            MSEL : MSEL_1
        );
        ral::write_reg!(ral::sai, sai, RCR3, RCE : 1);
        ral::write_reg!(
            ral::sai,
            sai,
            RCR4,
            FSD : FSD_1,
            FSP : FSP_1,
            FSE : FSE_1,
            MF : MF_1,
            SYWD : 32 - 1,
            FRSZ : 2 - 1
        );
        ral::write_reg!(ral::sai, sai, RCR5, WNW : 31, W0W : 31, FBT : 31);

        Sai { sai, pins}

    }

    /// Read out version as a tuple of (major, minor)
    pub fn read_version(sai: ral::sai::Instance<N>) -> (u32, u32) {
        let major = ral::read_reg!(ral::sai, sai, VERID, MAJOR);
        let minor = ral::read_reg!(ral::sai, sai, VERID, MINOR);
        (major, minor)
    }
}
