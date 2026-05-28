# Cyboard Imprint LED Map

Mapping mode uses `MAPPING_MODE = true` (compile-time).
Flash both halves with mapping firmware, then:

1. Boot — first LED lights up in dim cyan.
2. Identify the glowing key or decorative area.
3. Press that key (or any key if it's decorative) — advances to the next LED.
4. Record each LED in the table below. Repeat until all LEDs cycle back to start.
5. Count consecutive unlit indices to determine the real chain length.


Note: idk how to count columns/rows for thumb clusters, so i counted them last after "normal" rows and columns.
E.g. row 1 = top row on left half, row 6 = bottom row on left half,  column 1 = leftmost column on left half, column
6 = rightmost column on left half. Since thumb cluster for left half is "tilted", i still count it as 2 rows, 3 columns.

## Left / QWERTY Half

| LED Index | Physical Location | Row | Col | Notes            |
|-----------|-------------------|-----|-----|------------------|
| 0         | G                 | 3   | 6   |                  |
| 1         | B                 | 4   | 6   |                  |
| 2         | 6                 | 1   | 6   | ...              |
| 3         | T                 | 2   | 6   | ...              |
| 4         | F                 | 3   | 5   | ...              |
| 5         | V                 | 4   | 5   | ...              |
| 6         | 5                 | 1   | 5   | ...              |
| 7         | R                 | 2   | 5   | ...              |
| 8         | D                 | 3   | 4   | ...              |
| 9         | c                 | 4   | 4   | ...              |
| 10        | cmd               | 5   | 4   | ...              |
| 11        | 4                 | 1   | 4   | ...              |
| 12        | E                 | 2   | 4   | ...              |
| 13        | s                 | 3   | 3   |                  |
| 14        | x                 | 4   | 3   |                  |
| 15        | opt               | 5   | 3   | ...              |
| 16        | 3                 | 1   | 3   | ...              |
| 17        | w                 | 2   | 3   | ...              |
| 18        | a                 | 3   | 2   | ...              |
| 19        | z                 | 4   | 2   | ...              |
| 20        | 2                 | 1   | 2   | ...              |
| 21        | q                 | 2   | 2   | ...              |
| 22        | lshift            | 3   | 1   | ...              |
| 23        | lctrl             | 4   | 1   | ...              |
| 24        | 1                 | 1   | 1   | ...              |
| 25        | tab               | 2   | 1   | ...              |
| 26        | left              | 13  | 1   | thumb first row  |
| 27        | up                | 13  | 2   |                  |
| 28        | right             | 13  | 3   | ...              |
| 29        | fn                | 14  | 1   | thumb second row |
| 30        | down              | 14  | 2   | ...              |
| 31        | enter             | 14  | 3   | ...              |


Real chain length: 32

## Right / YUIOP Half

| LED Index | Physical Location | Row | Col | Notes                        |
|-----------|-------------------|-----|-----|------------------------------|
| 0         | H                 | 3   | 7   | 7th total, first on 2nd half |
| 1         | n                 | 4   | 7   |                              |
| 2         | 7                 | 1   | 7   | ...                          |
| 3         | y                 | 2   | 7   | ...                          |
| 4         | j                 | 3   | 8   | ...                          |
| 5         | m                 | 4   | 8   | ...                          |
| 6         | 8                 | 1   | 8   | ...                          |
| 7         | u                 | 2   | 8   | ...                          |
| 8         | k                 | 3   | 9   | ...                          |
| 9         | ,                 | 4   | 9   | ...                          |
| 10        | [                 | 5   | 9   | ...                          |
| 11        | 9                 | 1   | 9   | ...                          |
| 12        | i                 | 2   | 9   | ...                          |
| 13        | l                 | 3   | 10  |                              |
| 14        | .                 | 4   | 10  |                              |
| 15        | ]                 | 5   | 10  | ...                          |
| 16        | 0                 | 1   | 10  | ...                          |
| 17        | o                 | 2   | 10  | ...                          |
| 18        | ;                 | 3   | 11  | ...                          |
| 19        | /                 | 4   | 11  | ...                          |
| 20        | -                 | 1   | 11  | ...                          |
| 21        | p                 | 2   | 11  | ...                          |
| 22        | '                 | 3   | 12  | ...                          |
| 23        | ralt              | 4   | 12  | ...                          |
| 24        | =                 | 1   | 12  | ...                          |
| 25        | \                 | 2   | 12  | ...                          |
| 26        | pgdn              | 15  | 1   | thumb first row              |
| 27        | pgup              | 15  | 2   |                              |
| 28        | del               | 15  | 3   | ...                          |
| 29        | bksp              | 16  | 1   | thumb second row             |
| 30        | ins               | 16  | 2   | ...                          |
| 31        | space             | 16  | 3   | ...                          |

Real chain length: 32

## Decorative LEDs (not mapped to keys)
None, only onboard leds.
