# Cyboard Imprint Per-Key RGB Implementation Plan

## Goal

Turn the current Imprint RGB probe into a usable per-key RGB processor for the split Cyboard Imprint nRF52840 example.

The finished feature should:

- drive each half's WS2812 chain from `P0_08`/`PWM0`
- keep the existing split keyboard, BLE, USB, Vial, matrix, trackpad, and diagnostic LED behavior working
- render layer, host LED, connection, and optional battery state without blocking typing
- support per-key addressing once the physical LED order is mapped

This is an implementation plan, not a research plan. The only unresolved hardware fact is the exact LED index to physical key mapping.

## Current State

Implemented now:

- `src/imprint_rgb.rs` registers as an RMK polling processor.
- `src/central.rs` and `src/peripheral.rs` both initialize `PWM0` on `P0_08` with `SequencePwm`.
- The processor uses T1H = `0x8000 | 13`, T0H = `0x8000 | 7`, RES = `0x8000`, `end_delay = 799`.
- It renders a startup fill-wave then writes dim teal via a dirty-gated renderer.
- `README.md` documents this as an RGB probe.

Keep that working while adding per-key behavior. Do not replace it with RMK `[light]`; that service is for simple GPIO indicator LEDs, not addressable RGB.

## Constraints

- Keep brightness low during all bring-up. Start with values in the `0..8` range per channel.
- Keep the onboard blue diagnostic LED on `P0_30` unchanged.
- Do not use pins assigned to matrix scanning or the PMW3610 trackpad.
- Do not assume 64 LEDs is final. The current `LED_COUNT` is a probe value.
- Treat any QMK or Vial LED config as a hint only; verify it on the physical keyboard.
- The central and peripheral each drive their own local LED chain. Split state that already propagates through RMK should be used rather than inventing a second transport.

## Phase 1: Make The Probe Explicit And Bounded

Files:

- `examples/use_config/cyboard_imprint_nrf52840/src/imprint_rgb.rs`
- `examples/use_config/cyboard_imprint_nrf52840/README.md`

Tasks:

1. Rename internal probe constants so their purpose is obvious:
   - `LED_COUNT` -> `PROBE_LED_COUNT`
    - `FRAME_BYTES` -> derived from `PROBE_LED_COUNT`
2. Add a short comment beside the WS2812 PWM encoding:
    - `T1H = 0x8000 | 13` encodes a one bit (812.5 ns)
    - `T0H = 0x8000 | 7` encodes a zero bit (437.5 ns)
    - `RES = 0x8000` is the reset word
    - `end_delay = 799` extends reset to ~1 ms
3. Keep the existing all-LED layer/connection/Caps Lock rendering as the fallback mode.
4. Confirm the reset end_delay provides sufficient reset time for the full chain.
5. Update `README.md` only if behavior or counts change.

Acceptance:

- `cargo make uf2 --release` builds both normal UF2s.
- Both halves still light at low brightness after flashing.
- Base layer, layer 1, layer 2+, BLE advertising, and Caps Lock still change the visible color.

## Phase 2: Add LED Index Mapping Mode

Files:

- `src/imprint_rgb.rs`
- optionally `README.md` for operator instructions

Tasks:

1. Add a compile-time mapping mode to `ImprintRgb`.
   - Keep it local to the example.
   - Prefer a simple `const RGB_MAPPING_MODE: bool = false;` at the top of `imprint_rgb.rs` unless a repo pattern already exists for example-only flags.
2. In mapping mode, render exactly one LED index at a time.
   - Lit LED color: dim white or dim cyan.
   - All other LEDs: off.
   - Advance the index on a slow timer first, for example once every second.
3. Subscribe to `KeyboardEvent` after the timer mode works.
   - On a pressed key event, advance to the next LED.
   - Ignore release events.
   - Keep timer advancement disabled once key advancement is working, so mapping is controllable.
4. Add a `current_led` field to `ImprintRgb`.
5. Add an `encode_off()` or `clear_leds()` helper so mapping mode cannot leave stale colors in the frame.

Acceptance:

- With mapping mode off, behavior is identical to the current probe.
- With mapping mode on, only one LED is lit at a time.
- Pressing any key advances to the next LED without breaking normal typing or split communication.
- The highest tested LED index is recorded.

## Phase 3: Record The Physical LED Map

Files:

- new `examples/use_config/cyboard_imprint_nrf52840/led_map.md`
- later `src/imprint_rgb.rs`

Tasks:

1. Flash mapping mode to both halves.
2. Walk the full chain on the left/QWERTY half and record:
   - LED index
   - physical key or decorative location
   - matrix row/column if it is a key LED
   - notes for reversed order, underglow, thumb keys, or skipped indices
3. Repeat on the right/YUIOP half.
4. Stop increasing the index only after several consecutive unlit indices beyond the last visible LED.
5. Convert the notes into two tables:
   - `LEFT_LED_BY_KEY[row][col]`
   - `RIGHT_LED_BY_KEY[row][col]`
6. Use `Option<u8>` or a sentinel for keys without a known LED.
7. Keep decorative LEDs in a separate ordered list.

Acceptance:

- Every visible per-key LED has a documented index.
- Every mapped key can be lit by matrix row/column.
- Decorative LEDs, if any, are not mixed into the key map.
- The plan can name the real `LED_COUNT` for each half instead of using the probe count.

## Phase 4: Split The Renderer Into Data And Effects

Files:

- `src/imprint_rgb.rs`

Tasks:

1. Replace the probe-only whole-chain render path with a small renderer model:
   - `Rgb { r, g, b }`
   - fixed LED buffer sized to the verified LED count
   - key map table for this half
   - optional decorative LED table
2. Keep the whole-chain color renderer as `render_solid(color)`.
3. Add `render_key(row, col, color)` that:
   - looks up the LED index in the verified map
   - ignores unmapped keys
   - writes only that LED's GRB bytes
4. Add `render_layer_base()` that colors mapped key LEDs by active layer.
5. Add priority overlays in this order:
   - Caps Lock overrides mapped keys with dim red
   - BLE advertising blink overrides the chain with dim blue/off
   - low battery warning can be added later after battery state is available and verified
6. Keep PWM writes dirty-gated so unchanged frames are not written every poll.

Acceptance:

- Layer color is rendered through the verified map, not by assuming LED index equals matrix position.
- Caps Lock still has a clear visible override.
- Advertising blink still works.
- PWM writes remain bounded to the polling processor and do not add blocking work to matrix scanning.

## Phase 5: Add Reactive Per-Key Press Feedback

Files:

- `src/imprint_rgb.rs`

Tasks:

1. Subscribe to `KeyboardEvent` in normal mode.
2. Track a small per-key activity buffer:
   - key row
   - key column
   - remaining ticks or intensity
3. On key press:
   - add or refresh that key in the activity buffer
   - mark the frame dirty
4. On each poll:
   - decay active keys
   - render layer base first
   - overlay active pressed keys with a brighter but still conservative color
5. Keep the buffer small and fixed-size. Avoid heap allocation.

Acceptance:

- Pressing a mapped key lights that key.
- The effect decays without leaving stale LEDs lit.
- Unmapped keys do nothing visually but still type normally.
- Both halves show local key press effects.

## Phase 6: Add Keymap Controls

Files:

- `keyboard.toml`
- `keyboard_ble_only.toml`
- `src/imprint_rgb.rs`

Tasks:

1. Add user actions only after mapped rendering works.
2. Implement controls in the processor through `KeyboardEvent` plus keymap `User` actions if the current RMK API exposes the user action ID through events.
3. Minimum controls:
   - RGB on/off
   - brightness up
   - brightness down
   - mode cycle
4. Keep defaults conservative:
   - RGB enabled
   - low brightness
   - static layer mode
5. Persist settings only after volatile controls are stable.

Acceptance:

- Controls work over normal USB+BLE firmware.
- Controls do not break BLE pairing controls already on layer 2.
- Brightness bounds prevent accidental high-current testing.

## Phase 7: Cleanup And Documentation

Files:

- `README.md`
- `per_key.md`
- `led_map.md`
- `src/imprint_rgb.rs`

Tasks:

1. Move mapping-mode instructions into `README.md` or keep them in `led_map.md`.
2. Update `per_key.md` with completed phases and any hardware findings.
3. Remove stale probe language once per-key rendering is the default.
4. Keep a short troubleshooting section:
   - no LEDs light
   - wrong colors
   - first N LEDs light but later LEDs do not
   - one half lights and the other does not
   - typing works but RGB does not update

Acceptance:

- A new maintainer can rebuild, flash, map, and verify RGB without reading commit history.
- The current verified LED count and maps are documented.
- Temporary mapping code is disabled by default.

## Build And Test Matrix

Run after each implementation phase:

```shell
cargo make uf2 --release
```

Run when touching BLE-only config or comparing transport behavior:

```shell
cargo make uf2-ble-only --release
```

Manual checks:

- Left/QWERTY half flashes and types as central.
- Right/YUIOP half flashes and sends keys as peripheral.
- Split reconnects after power cycle.
- USB typing still works on the central normal firmware.
- BLE typing still works after pairing.
- Caps Lock LED state reaches both halves.
- Layer changes reach both halves.
- RGB never uses high brightness during bring-up.

## Definition Of Done

Per-key RGB is done when:

- the real LED count is known for each half
- matrix row/column to LED index maps are committed
- the processor renders mapped keys rather than only whole-chain colors
- Caps Lock, layer, connection, and key press effects work on both halves
- normal and BLE-only firmware still build
- README instructions match the actual flashing and verification flow
