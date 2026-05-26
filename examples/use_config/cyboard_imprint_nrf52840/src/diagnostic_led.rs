use embassy_nrf::gpio::Output;
use rmk::event::{
    CentralConnectedEvent, ClearPeerEvent, ConnectionStatusChangeEvent, KeyboardEvent, KeyboardEventPos,
    PeripheralConnectedEvent,
};
use rmk::macros::processor;
use rmk::types::ble::BleState;

#[processor(
    subscribe = [
        ConnectionStatusChangeEvent,
        PeripheralConnectedEvent,
        CentralConnectedEvent,
        ClearPeerEvent,
        KeyboardEvent
    ],
    poll_interval = 100
)]
pub struct DiagnosticLed {
    pin: Output<'static>,
    tick: u16,
    pulse_edges: u8,
    ble_state: BleState,
    host_connected: bool,
    split_connected: bool,
}

impl DiagnosticLed {
    pub fn new(pin: Output<'static>) -> Self {
        Self {
            pin,
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
        self.host_connected = status.ble.state == BleState::Connected;

        if status.ble.state == BleState::Connected && old_state != BleState::Connected {
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
        self.flash(if event.connected { 2 } else { 8 });
    }

    async fn on_clear_peer_event(&mut self, _event: ClearPeerEvent) {
        self.flash(7);
    }

    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if !event.pressed {
            return;
        }

        let KeyboardEventPos::Key(pos) = event.pos else {
            return;
        };

        match (pos.row, pos.col) {
            (0, 1) => self.flash(5), // CLR_BT
            (0, 2) => self.flash(1), // BT0
            (0, 3) => self.flash(2), // BT1
            (0, 5) => self.flash(3), // BT2
            (0, 6) | (0, 7) => self.flash(4), // NEXT_BT/PREV_BT
            (1, 2) => self.flash(6), // SWITCH
            (1, 3) => self.flash(7), // CLR_PEER pressed; actual clear also flashes after hold
            _ => {}
        }
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

        if self.host_connected {
            self.set_led(true);
            return;
        }

        match self.ble_state {
            BleState::Advertising => self.set_led(self.tick % 10 < 2),
            BleState::Connected => self.set_led(true),
            BleState::Inactive if self.split_connected => self.set_led(self.tick % 20 == 0 || self.tick % 20 == 2),
            BleState::Inactive => self.set_led(self.tick % 40 == 0),
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
