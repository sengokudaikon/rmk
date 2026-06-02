#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

mod diagnostic_led;
mod imprint_rgb;

#[rmk_peripheral(id = 0)]
mod keyboard_peripheral {
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
        let mut config = ::embassy_nrf::pwm::Config::default();
        config.prescaler = ::embassy_nrf::pwm::Prescaler::Div1;
        config.max_duty = 20;
        config.sequence_load = ::embassy_nrf::pwm::SequenceLoad::Common;
        let pwm = ::embassy_nrf::pwm::SequencePwm::new_1ch(p.PWM0, p.P0_08, config)
            .expect("PWM0");
        crate::imprint_rgb::ImprintRgb::new(pwm, false)
    }
}
