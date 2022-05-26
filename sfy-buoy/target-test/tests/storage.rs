#![no_std]
#![no_main]

use ambiq_hal as hal;
use defmt_rtt as _;
use panic_probe as _; // memory layout + panic handler

#[defmt_test::tests]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    #[allow(unused)]
    use defmt::{assert, assert_eq, info};
    use embedded_hal::spi;
    use embedded_sdmmc::SdMmcSpi;
    use hal::prelude::*;
    use hal::spi::{Freq, Spi};

    use sfy::storage::Storage;

    struct State {
        // note: Notecarrier<hal::i2c::Iom2>,
        #[allow(unused)]
        delay: hal::delay::Delay,
        #[allow(unused)]
        rtc: hal::rtc::Rtc,

        storage: Storage,
    }

    #[init]
    fn setup() -> State {
        defmt::debug!("Setting up peripherals");
        let core = hal::pac::CorePeripherals::take().unwrap();
        let mut dp = hal::pac::Peripherals::take().unwrap();
        let pins = hal::gpio::Pins::new(dp.GPIO);

        let rtc = hal::rtc::Rtc::new(dp.RTC, &mut dp.CLKGEN);
        let delay = hal::delay::Delay::new(core.SYST, &mut dp.CLKGEN);

        defmt::info!("Setting up SPI");
        let spi = Spi::new(
            dp.IOM0,
            pins.d12,
            pins.d13,
            pins.d11,
            Freq::F100kHz,
            spi::MODE_0,
        );

        let cs = pins.a14.into_push_pull_output();
        let storage = Storage::open(spi, cs).unwrap();

        State { delay, rtc, storage }
    }

    #[test]
    fn initialize_storage(s: &mut State) {
        defmt::info!("current id: {:?}", s.storage.current_id());
        assert_eq!(s.storage.current_id(), Some(0), "tests run on card with data");
    }

    #[test]
    fn write_id(s: &mut State) {
        s.storage.write_id().unwrap();
        assert_eq!(s.storage.read_id().unwrap(), 0);

        s.storage.set_id(1);
        s.storage.write_id().unwrap();
        assert_eq!(s.storage.read_id().unwrap(), 1);

        s.storage.set_id(0);
        s.storage.write_id().unwrap();
        assert_eq!(s.storage.read_id().unwrap(), 0);
    }
}
