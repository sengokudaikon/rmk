#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

mod diagnostic_led;
mod imprint_rgb;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
    add_interrupt!(SPIM3 => ::embassy_nrf::spim::InterruptHandler<::embassy_nrf::peripherals::SPI3>;);

    #[register_processor(poll)]
    fn diagnostic_led() -> crate::diagnostic_led::DiagnosticLed {
        crate::diagnostic_led::DiagnosticLed::new(
            ::embassy_nrf::gpio::Output::new(
                p.P0_30,
                ::embassy_nrf::gpio::Level::Low,
                ::embassy_nrf::gpio::OutputDrive::Standard,
            ),
            crate::diagnostic_led::DiagnosticRole::Peripheral,
        )
    }

    #[register_processor(poll)]
    fn imprint_rgb() -> crate::imprint_rgb::ImprintRgb {
        let mut config = ::embassy_nrf::spim::Config::default();
        config.frequency = ::embassy_nrf::spim::Frequency::M4;
        crate::imprint_rgb::ImprintRgb::new(::embassy_nrf::spim::Spim::new_txonly_nosck(
            p.SPI3, Irqs, p.P0_08, config,
        ))
    }
}
