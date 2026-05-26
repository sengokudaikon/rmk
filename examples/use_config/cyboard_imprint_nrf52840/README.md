# Cyboard Imprint RMK Bring-Up

This is an RMK bring-up for a split Cyboard Imprint on nRF52840 Assimilator BLE boards.

## Build Firmware

Build the normal USB+BLE split UF2s:

```shell
cargo make uf2 --release
```

This writes:

- `rmk-central.uf2`
- `rmk-peripheral.uf2`

For BLE transport diagnostics, build the no-USB variant:

```shell
cargo make uf2-ble-only --release
```

This writes:

- `rmk-central-ble-only.uf2`
- `rmk-peripheral-ble-only.uf2`

## Flashing Guide

The intended Imprint split assignment matches the ZMK shield:

- left/QWERTY half: `central`
- right/YUIOP half: `peripheral`

Use the normal USB+BLE firmware as the primary test build:

1. Build with `cargo make uf2 --release`.
2. Put the left/QWERTY half into bootloader.
3. Copy `rmk-central.uf2` to the bootloader drive.
4. Put the right/YUIOP half into bootloader.
5. Copy `rmk-peripheral.uf2` to the bootloader drive.
6. Reboot/power-cycle both halves.

If a half behaves strangely after repeated experiments, flash the board restore/reset UF2 first, reboot back into bootloader, then flash the matching RMK UF2 above.

The BLE-only files are experimental diagnostics only:

- left/QWERTY half: `rmk-central-ble-only.uf2`
- right/YUIOP half: `rmk-peripheral-ble-only.uf2`

The BLE-only build disables USB transport and will not type over USB. In current bring-up it may behave differently from the normal firmware, so do not treat BLE-only failures as proof that the matrix or split wiring is bad.

## LED Diagnostics

The onboard blue LED is used for bring-up diagnostics:

- startup: several quick flashes
- sparse flash: firmware is alive/idle
- central key press: one quick flash
- central advertising to host: repeating blink
- central host connected: solid on
- split connect/disconnect events: short flash bursts

If the normal firmware makes both halves blink and central key presses flash, the matrix scan path is alive.

## Layout

Base layer:

- Top row is `= 1 2 3 4 5` on the left and `6 7 8 9 0 -` on the right.
- Letter rows are `Tab QWERT`, `Shift ASDFG`, `Ctrl ZXCVB`, mirrored on the right.
- Thumb cluster is Enter/Delete/Escape and Grave/Backspace/Space, with MO(1)/MO(2) on the lower thumb keys.

Layer 1 puts F1-F11 on the number row and keeps simple nav/numpad keys.

Layer 2 has Bluetooth controls on the left number row: clear current bond, BT0, BT1, BT2, next profile, previous profile.
