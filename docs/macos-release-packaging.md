# macOS release packaging

DustFril's macOS production bundle creates an application bundle and a DMG:

- `target/release/bundle/macos/DustFril.app`
- `target/release/bundle/dmg/DustFril_<version>_aarch64.dmg`

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
hdiutil verify target/release/bundle/dmg/DustFril_<version>_aarch64.dmg
hdiutil attach -nobrowse -readonly target/release/bundle/dmg/DustFril_<version>_aarch64.dmg
```

Confirm that the mounted volume contains `DustFril.app`, an
`Applications` link to `/Applications`, and the expected volume icon. Detach
the volume with `hdiutil detach <device>` after verification.

## v0.1.0 release policy

The first public release is built and validated for Apple Silicon only
(`aarch64`). It is distributed through the
[DustFril GitHub Releases page](https://github.com/FrilLab/dustfril/releases)
as `DustFril_0.1.0_aarch64.dmg` with a matching `.sha256` file.

DustFril v0.1.0 is currently an unsigned/not-notarized early release. macOS
may show a security warning on first launch. Do not describe this build as
signed or notarized; signed and notarized distribution is tracked separately
in #161.

After the release is public, validate the downloaded artifact rather than only
the local build:

1. Download the DMG and its `.sha256` file from the GitHub Release.
2. Run `shasum -a 256 -c DustFril_0.1.0_aarch64.dmg.sha256`.
3. Run `hdiutil verify DustFril_0.1.0_aarch64.dmg` and mount it read-only.
4. Copy `DustFril.app` to `Applications` or a disposable test location and
   launch it, acknowledging the unsigned/not-notarized warning if shown.
5. In a disposable workspace fixture, analyze the workspace, perform one
   Trash cleanup, verify Activity History, and confirm unrelated files are
   unchanged.
6. Detach the DMG and confirm no release DMG remains mounted.
