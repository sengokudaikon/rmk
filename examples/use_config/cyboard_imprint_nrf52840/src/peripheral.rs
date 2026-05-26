#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

mod diagnostic_led;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
    #[register_processor(poll)]
    fn diagnostic_led() -> crate::diagnostic_led::DiagnosticLed {
        crate::diagnostic_led::DiagnosticLed::new(::embassy_nrf::gpio::Output::new(
            p.P0_30,
            ::embassy_nrf::gpio::Level::Low,
            ::embassy_nrf::gpio::OutputDrive::Standard,
        ), crate::diagnostic_led::DiagnosticRole::Peripheral)
    }
}
