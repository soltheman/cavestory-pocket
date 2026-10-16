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

## Touch controls

Options > Controls > Touch controls sets the buttons' opacity and size, and "Move buttons..." lets you
drag each one into place. The layout is saved in `user/settings.json` (`touch_layout`).

To try the phone controls on a PC build, set `CAVESTORY_TOUCH=1` (the mouse acts as a finger) and
start with `--window-width 1266 --window-height 585` for an iPhone-shaped window; `CAVESTORY_DATA_DIR`
points at a copy of the game's `data` folder.

## Icons

`make_curly_icon.ps1` draws the Curly icon (`curly-icon-1024.png`, original pixel art), and
`make_icons.ps1` turns it into the sizes in `icons/`.
