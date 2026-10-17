# Cave Story+ on iPhone

An iOS build of doukutsu-rs. The app contains only the engine: the Cave Story+ game files come from
your own copy and are never put in the IPA or a release.

## Build

Run the `ios-release` workflow in GitHub Actions (manual only). It builds on a macOS runner and uploads
an unsigned `CaveStoryPocket.ipa` as an artifact, or as a pre-release when given a tag. Sideload it
the same way as Bo1ZPocket.

## Game files

doukutsu-rs reads Cave Story+ data in the original (pre-2026) format, e.g. the Epic Games Store copy.
The Steam 2026 update's format is not supported yet (upstream issue #365).

1. Open the app once so its folder appears in the Files app.
2. Copy the whole `data` folder from the PC install (for example
   `C:\Program Files\Epic Games\CaveStoryPlus\data`) into Files > On My iPhone > Cave Story+.
3. Close and reopen the app.

Saves and settings live in `user` next to `data`, and logs in `logs`, all visible in the Files app.

## Resume after iOS closes the game

Leaving the app mid-game writes `user/Resume<slot>.dat` (only when the player is in control, with no
cutscene, credits or boss fight running). If iOS closes the app while it's away, the title screen offers
**Resume** to continue from there; coming back normally deletes the file. Rooms restart as when loading a
save. On a PC with `CAVESTORY_TOUCH=1`, minimising the window does the same, for testing.

## Touch controls

Options > Controls > Touch controls sets the buttons' opacity, size, icons (Classic, Bold, Text),
colour, backing shape, d-pad look (arrows or a stick), whether the weapon-switch buttons show and
whether the Map button shows, whether the phone vibrates (a tap when hurt, light taps during quakes) and
whether pressing a button ticks (`src/framework/haptics.rs`), and
"Move buttons..." lets you drag each one into place. The layout is saved in `user/settings.json`
(`touch_layout`). `make_touch_art.ps1` regenerates the button art (`touch_icons.png`, `touch_plate.png`).

A game controller (MFi, Xbox, PlayStation) works as player 1 as soon as it connects, and the touch
buttons hide while one is. The Controls menu leaves out keyboard settings with touch controls on;
player selection, the controller list and button rebinding show only while a controller is connected.

To try the phone controls on a PC build, set `CAVESTORY_TOUCH=1` (the mouse acts as a finger) and
start with `--window-width 1266 --window-height 585` for an iPhone-shaped window; `CAVESTORY_DATA_DIR`
points at a copy of the game's `data` folder.

## Icons

The released app uses doukutsu-rs's icon; `make_icons.ps1` regenerates `icons/` from
`res/macos/512x512@2x.png`.

For Curly as the icon on your own phone, run `curly_icon_ipa.ps1 -Ipa <downloaded ipa>`. It takes
her portrait from your game files and writes a personal `-curly.ipa` to sideload. Game art never
goes in this repo or its releases.
