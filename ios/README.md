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

## Icons

`make_icons.ps1` regenerates `icons/` from `res/macos/512x512@2x.png`.
