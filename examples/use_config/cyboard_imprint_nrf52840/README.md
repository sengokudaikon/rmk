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

## Flash

Flash central to the left half and peripheral to the right half. The BLE-only files are only for checking whether USB transport preference is hiding BLE behavior; they will not type over USB.

## Layout

Base layer:

- Top row is `= 1 2 3 4 5` on the left and `6 7 8 9 0 -` on the right.
- Letter rows are `Tab QWERT`, `Shift ASDFG`, `Ctrl ZXCVB`, mirrored on the right.
- Thumb cluster is Enter/Delete/Escape and Grave/Backspace/Space, with MO(1)/MO(2) on the lower thumb keys.

Layer 1 puts F1-F11 on the number row and keeps simple nav/numpad keys.

Layer 2 has Bluetooth controls on the left number row: clear current bond, BT0, BT1, BT2, next profile, previous profile.
