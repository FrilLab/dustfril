# macOS release packaging

DustFril's macOS production bundle creates an application bundle and a DMG:

- `target/release/bundle/macos/dustfril-tauri.app`
- `target/release/bundle/dmg/dustfril-tauri_<version>_aarch64.dmg`

## DMG build failure on a local macOS session

The Tauri 2.11.2 bundler uses its generated `bundle_dmg.sh` script. The script
uses `hdiutil` to create and mount an interstitial image, then runs
`/usr/bin/osascript` to ask Finder to save the DMG window layout.

Run the normal build with verbose output when investigating a failure:

```sh
cd apps/tauri
npm run tauri build -- --verbose
```

If the output reaches `Running AppleScript to make Finder stuff pretty` and
then reports:

```text
execution error: Not authorized to send Apple events to Finder. (-1743)
Failed running AppleScript
```

the failure is macOS Automation/TCC permission for the process invoking
`osascript`. It is not an `hdiutil` failure, a stale mounted volume, a
filesystem permission failure, signing/notarization, or a DustFril icon/config
failure. The script ejects the interstitial image before returning the failure.

On a release workstation where Finder window positioning is required, grant
the invoking terminal/automation host permission to control Finder in System
Settings > Privacy & Security > Automation, then rerun the normal command.

## Headless or CI release workaround

For a headless/non-interactive macOS release environment, retain DMG generation
but skip only the Finder AppleScript decoration step:

```sh
cd apps/tauri
CI=true TAURI_BUNDLER_DMG_IGNORE_CI=false npm run tauri build -- --verbose
```

This still runs `hdiutil`, adds the Applications shortcut, copies the volume
icon, fixes permissions, ejects the interstitial image, and compresses the
final DMG. The resulting DMG does not contain Finder-saved icon positioning or
window cosmetics. Do not set `TAURI_BUNDLER_DMG_IGNORE_CI=true` in a headless
environment; that forces the Finder AppleScript and recreates the TCC failure.

After either build, verify the artifact before release:

```sh
hdiutil verify target/release/bundle/dmg/dustfril-tauri_<version>_aarch64.dmg
hdiutil attach -nobrowse -readonly target/release/bundle/dmg/dustfril-tauri_<version>_aarch64.dmg
```

Confirm that the mounted volume contains `dustfril-tauri.app`, an
`Applications` link to `/Applications`, and the expected volume icon. Detach
the volume with `hdiutil detach <device>` after verification.
