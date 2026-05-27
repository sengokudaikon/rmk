use embassy_nrf::gpio::Output;
use rmk::event::{
    CentralConnectedEvent, ConnectionStatusChangeEvent, PeripheralConnectedEvent,
};
use rmk::macros::processor;
use rmk::types::ble::BleState;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DiagnosticRole {
    Central,
    Peripheral,
}

#[processor(
    subscribe = [
        ConnectionStatusChangeEvent,
        PeripheralConnectedEvent,
        CentralConnectedEvent,
    ],
    poll_interval = 250
)]
pub struct DiagnosticLed {
    pin: Output<'static>,
    role: DiagnosticRole,
    tick: u16,
    pulse_edges: u8,
    ble_state: BleState,
    host_connected: bool,
    split_connected: bool,
}

impl DiagnosticLed {
    pub fn new(pin: Output<'static>, role: DiagnosticRole) -> Self {
        Self {
            pin,
            role,
            tick: 0,
            pulse_edges: 12,
            ble_state: BleState::Inactive,
            host_connected: false,
            split_connected: false,
        }
    }

    async fn on_connection_status_change_event(&mut self, event: ConnectionStatusChangeEvent) {
        let status = event.0;
        let old_state = self.ble_state;
        self.ble_state = status.ble.state;
        if self.role == DiagnosticRole::Central {
            self.host_connected = status.ble.state == BleState::Connected;
        }

        if self.role == DiagnosticRole::Central
            && status.ble.state == BleState::Connected
            && old_state != BleState::Connected
        {
            self.flash(3);
        } else if status.ble.state == BleState::Advertising && old_state != BleState::Advertising {
            self.flash(2);
        }
    }

    async fn on_peripheral_connected_event(&mut self, event: PeripheralConnectedEvent) {
        self.split_connected = event.connected;
        self.flash(if event.connected { 2 } else { 8 });
    }

    async fn on_central_connected_event(&mut self, event: CentralConnectedEvent) {
        self.split_connected = event.connected;
        self.host_connected = false;
        self.flash(if event.connected { 2 } else { 8 });
    }

    async fn poll(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if self.pulse_edges > 0 {
            if self.tick % 2 == 0 {
                self.pulse_edges -= 1;
                self.set_led(self.pulse_edges % 2 == 1);
            }
            return;
        }

        if self.role == DiagnosticRole::Central && self.host_connected {
            self.set_led(true);
            return;
        }

        match (self.role, self.ble_state, self.split_connected) {
            (DiagnosticRole::Central, BleState::Advertising, _) => self.set_led(self.tick % 10 < 2),
            (DiagnosticRole::Central, BleState::Connected, _) => self.set_led(true),
            (DiagnosticRole::Central, BleState::Inactive, true) => {
                self.set_led(self.tick % 20 == 0 || self.tick % 20 == 2)
            }
            (DiagnosticRole::Central, BleState::Inactive, false) => self.set_led(self.tick % 40 == 0),
            (DiagnosticRole::Peripheral, _, true) => self.set_led(self.tick % 20 == 0 || self.tick % 20 == 2),
            (DiagnosticRole::Peripheral, _, false) => self.set_led(self.tick % 40 == 0),
        }
    }

    fn flash(&mut self, count: u8) {
        self.pulse_edges = count.saturating_mul(2);
        self.tick = 1;
        self.set_led(true);
    }

    fn set_led(&mut self, on: bool) {
        if on {
            self.pin.set_high();
        } else {
            self.pin.set_low();
        }
    }
}
