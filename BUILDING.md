# Building ducktales locally

The source implements shared native desktop/browser/headless Game behavior.
Use Rust1.90.0 and port/Cargo.lock. These platform recipes are instructions for a
human developer, not evidence of device execution. Stage4 checks compile desktop
and release Wasm headlessly; stage5 owns two fresh whole-game QA passes.
Physical-device auditing and Android integration require a separate request.

For desktop builds, build on the target operating system and CPU architecture. Install Rust through
[rustup](https://rustup.rs/), use Rust 1.90.0 and retain port/Cargo.lock.

## Preserve and transfer source

Keep port/src/, Cargo.toml, Cargo.lock, rust-toolchain.toml and any authored
extensions. Retain the exact edited ROMAtlas source, including uncommitted
shared-library changes; Cargo.lock does not freeze a path dependency.
Desktop widescreen also requires the local ROMAtlas `Game::display_aspect_ratio`
addition described in [library-changes.md](library-changes.md). Transfer that
runtime source with the game. Browser builds retain the published runtime pin;
the desktop-only trait implementation is excluded from Wasm builds.

The generated dependency initially points to:

```text
/root/romatlas/crates/romatlas-port
```

Exclude target/ directories when copying; Cargo recreates their build products.
Reference archives, extracted assets and generated binaries are unnecessary for
running the native port. Supply your own matching ROM on the destination.

For this recipient layout:

```text
games/ducktales/port/
games/romatlas/
```

change port/Cargo.toml to use:

```toml
romatlas-port = { path = "../../romatlas/crates/romatlas-port" }
```

From a Mac/Linux terminal, substitute the server and workspace paths:

```sh
rsync -avz --progress --exclude='target/' user@server:/path/to/port-workspace/port/ ./games/ducktales/port/
```

Transfer the edited ROMAtlas source too, preserving its root Cargo files and
source layout; exclude target/, .git/, test ROMs and generated/local game assets.
The native port needs the shared library, not the disassembly reference project.
Use Cargo directly on the recipient: generated run.py/workspace.json still record
the preparation machine's toolkit paths and frozen-reference inventory.

## macOS

Install Xcode command-line tools, Rust and SDL:

```sh
xcode-select --install
rustup toolchain install 1.90.0
brew install sdl2-compat
```

Homebrew currently supplies the SDL2 API through
[sdl2-compat](https://formulae.brew.sh/formula/sdl2-compat), backed by SDL3.
In the same Terminal session used to build:

```sh
export LIBRARY_PATH="$(brew --prefix)/lib${LIBRARY_PATH:+:$LIBRARY_PATH}"
cd ~/games/ducktales/port
cargo +1.90.0 build --release --locked --features desktop
./target/release/ducktales-port --rom "/full/path/to/your/game.nes"
```

If the linker reports `library 'SDL2' not found`, repeat the LIBRARY_PATH export
in that Terminal session and check `brew list sdl2-compat`. Rust and SDL libraries
must use the same architecture; use native arm64 tooling on Apple Silicon.
You can add the export to your shell startup file for future builds.
Reference: [Rust SDL2 macOS linking](https://github.com/Rust-SDL2/rust-sdl2#macos).

## Linux (Ubuntu/Debian)

```sh
sudo apt-get install build-essential libsdl2-dev
rustup toolchain install 1.90.0
cd ~/games/ducktales/port
cargo +1.90.0 build --release --locked --features desktop
./target/release/ducktales-port --rom "/full/path/to/your/game.nes"
```

Desktop launch requires a graphical session. Do not launch it on the headless
server. Other distributions require equivalent C build tools and SDL2 development
libraries.
Reference: [Rust SDL2 Linux requirements](https://github.com/Rust-SDL2/rust-sdl2#linux).

## Windows x64 (MSVC)

Install Rust with the MSVC toolchain and Visual Studio C++ build tools/Windows SDK
as described in the [rustup Windows guide](https://rust-lang.github.io/rustup/installation/windows.html).
Download an SDL2 MSVC development archive (SDL2-devel-2.x-VC.zip) from
[official SDL2 releases](https://github.com/libsdl-org/SDL/releases?q=SDL2&expanded=true).
Locate its lib/x64 folder containing SDL2.lib and SDL2.dll.

In PowerShell, adjust the source and SDL paths:

```powershell
rustup toolchain install 1.90.0
Set-Location C:\games\ducktales\port
$sdlLib = "C:\libraries\SDL2\lib\x64"
$env:LIB = "$sdlLib;$env:LIB"
cargo +1.90.0 build --release --locked --features desktop
Copy-Item "$sdlLib\SDL2.dll" .\target\release\SDL2.dll
.\target\release\ducktales-port.exe --rom "C:\games\game.nes"
```

Use forward slashes in the Cargo.toml dependency path. Keep SDL2.dll beside the
executable. This recipe is for Windows x64; other targets need matching libraries.
Reference: [Rust SDL2 Windows MSVC linking](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc).

## Android (APK integration required)

The generated project currently has a Rust CLI and an optional desktop SDL2
shell. It does not include an Android Activity, JNI entry point, Gradle project,
ROM picker or touch controls. The steps below describe the build integration a
port developer may add in separately requested work; installing an Android target alone does not produce a
playable APK. Keep the existing desktop/headless entry points working.

### Toolchain and native-code check

Install Android Studio and, through SDK Manager, Android SDK Platform-Tools,
a target SDK, NDK (Side by side) and CMake. Use the JDK required by the chosen
Android Gradle Plugin. Record exact SDK/NDK/JDK/Gradle/CMake versions in the
finished guide and pin the NDK with Gradle's `ndkVersion`. See the official
[NDK setup instructions](https://developer.android.com/studio/projects/install-ndk).

Start with `arm64-v8a` (`aarch64-linux-android`) for ARM64 phones. An x86_64
emulator needs `x86_64-linux-android` and matching SDL libraries. Install the Rust
targets and check the device-independent code from port/:

```sh
rustup target add --toolchain 1.90.0 aarch64-linux-android
cargo +1.90.0 check --release --locked --target aarch64-linux-android
```

This checks Rust code without linking SDL or packaging an app. It does not
validate Android execution. Native linking additionally needs the selected NDK's
Clang linker and ABI/API configuration; use one minimum API consistently across
Rust, SDL and Gradle. Host Linux SDL libraries cannot be used for Android.

### Add the Android application

1. Copy the `android-project` skeleton from a pinned **SDL2** source release into
   an android/ directory beside port/. Use the Java glue and native SDL sources
   from that same release. Follow the
   [SDL2 Android guide](https://github.com/libsdl-org/SDL/blob/SDL2/docs/README-android.md),
   rather than SDL3 instructions. Record the release and preserve its notices.
2. Move reusable game construction into a Rust library, retaining the desktop
   CLI. Add an Android `cdylib` target and an exported C-compatible `SDL_main`
   entry point, or a C shim that calls Rust. Match SDLActivity's library names
   and entry point, contain Rust panics at the FFI boundary, and return errors
   to the Android UI instead of terminating the process through the CLI.
3. Build SDL2 for each selected ABI with the skeleton's NDK/CMake integration.
   Configure Cargo's target linker and SDL search path for those outputs; wire
   the Rust build into Gradle so changes rebuild the native library. Package
   both the game shared library and its native dependencies for each ABI.
   Keep Cargo.lock and use `--release --locked` for Cargo invocations. Update
   this guide with the actual commands, library names and output locations.
4. Add a ROM selection screen using Android's
   [Storage Access Framework](https://developer.android.com/training/data-storage/shared/documents-files).
   A `content://` URI is not a filesystem path: read it through ContentResolver
   and copy it into bounded app-private storage before calling `Rom::open`,
   preserving SHA-256 validation. Keep settings and asset overrides in writable
   app storage. Never bundle the ROM or extracted original assets in the APK.
5. Adapt the shell for pause/resume, surface recreation, audio interruption,
   Android Back, safe screen areas and input rearming. Add usable touch controls
   and a settings/menu route without F1, or explicitly require a controller.
   The desktop shell alone does not supply these mobile interactions.

### Optional later build, install and device audit

After the application integration above exists, run from android/:

```sh
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

On Windows use `gradlew.bat`. Device play, settings/input/lifecycle/audio testing
belongs to a later explicitly requested device audit. Record any user-reported
results separately from the implementation's unit-test/build results. Cross-compilation and
APK assembly do not establish device behavior; no Android build or device test
has been performed for this port. Release signing, keystore management and
store packaging require a separate release setup; keep signing secrets local.

## Headless builds and checks

SDL2 is optional. Without the desktop feature, build and run device-free:

```sh
cargo +1.90.0 build --release --locked
./target/release/ducktales-port --rom "/full/path/to/your/game.nes" --headless 120
cargo +1.90.0 test --locked
```

Windows uses the .exe suffix. README.md documents controls and manifests.
In the original full workspace, `python3 run.py check` runs formatting, clippy,
headless tests and desktop compilation without launching graphics.

## Validation status

The required stage4 `python3 run.py check` passed formatting, all-target desktop
clippy with warnings denied, 1411 source/integration tests, desktop compilation,
Wasm clippy with warnings denied and release Wasm compilation. Exact results:
checks/implementation/stage4-check-final.json and stage4-check-final-gates.log.

No desktop window, browser play, other operating-system build, Android build or
physical device was tested on this headless server. Focused owner tests exercise
source semantics and shared joins; they do not establish whole-game fidelity.
Stage5 results remain in debug/qa.json, separate from implementation accounting.

## Later packaging

Build automation, CI, app bundles and installers can be applied to existing
source ports later. Retain exact port/library source and lockfiles as their
inputs. Bundling SDL/runtime dependencies, signing/notarization, immutable
library pinning and clean-machine release testing are separate later work.
These development instructions do not authorize publishing or distributing games.

## Browser / WebAssembly

For this native port, game logic and ROM asset extraction are shared
between SDL2 desktop and Canvas/Web Audio frontends. After toolkit setup:

```sh
python3 run.py web-build
python3 run.py web --port 8000
# Optional server-hosted replacement/additional assets:
python3 run.py web --assets /path/to/assets --port 8000
```

The default bind address is `0.0.0.0`; use `--host 127.0.0.1` for loopback or
`--host ADDRESS` for another interface. Open the printed URL and press Start for audio/gameplay. The configured
ROM must remain on the server; it is fetched, identity-checked and decoded at
startup, never embedded into the bundle. `web-dist/` contains generated Wasm,
bindings and shell only. Original assets must stay ROM-backed. Keep generated
files local. Existing workspaces are not upgraded automatically.

Browser settings support keyboard/standard gamepad rebinding, four player device
assignments, volume, scaling, dimensions and fullscreen. Settings and versioned
battery saves use browser localStorage keyed by ROM hash; keep the same server origin,
browser and profile to retain them. Start resumes after Pause; closing Settings resumes an already started game automatically. Focus loss does not pause the browser
game; browser background throttling can still limit updates. Reset reloads the
game and returns to the Start prompt, preserving settings and original battery
saves. No touch/mobile controls are provided. This port adds browser save states as described below. This port has not been runtime-tested in Chromium or Firefox. Builds are single-threaded and require no cross-origin
isolation. See the toolkit's docs/porting.md for the save contract and test setup.

The original cartridge has no battery; this Game uses the default no-save hooks.
Settings persist independently. Debug replay checkpoints are developer artifacts,
not cartridge saves. Required checks include locked Wasm compilation; toolkit
synthetic browser tests are separate from game fidelity work.

See README.md, EXTENSIONS.md and debug/api.md for shared Game controls, local
asset delivery, original-only manifests and deterministic debug/checkpoint APIs.

## GitHub Pages deployment

`.github/workflows/pages.yml` builds and deploys the browser game on every push
to `main`, or via Actions → Build and deploy browser port → Run workflow.
In repository Settings → Pages, select **GitHub Actions** as the build source
before the first deployment. The expected address is
`https://weisserw.github.io/ducktales-port/`.

The workflow checks out ROMAtlas at the exact revision named in the workflow,
installs Rust 1.90.0 and wasm-bindgen 0.2.129, and builds with Cargo.lock.
Both repositories must be siblings (`ducktales/` and `romatlas/`); Cargo uses
that relative dependency path. Update the pinned toolkit revision deliberately
when adopting shared runtime changes.

To build the same static site locally with that layout:

```sh
python3 scripts/build-pages.py
python3 -m http.server 8000 --directory web-dist
```

Unlike the development server bundle alone, this command adds `rom` and
`config.json` to `web-dist/`, verifies the ROM against `workspace.json`, and
makes the generated shell URLs relative for project subpaths. The published
site includes the tracked `input/game.rom`, which browsers download at startup.
Only `web-dist/` is uploaded; reference trees, QA evidence and source are excluded.
Generated files remain ignored. A failed build does not deploy a replacement site.
`run.py` finds the sibling ROMAtlas checkout by default; `ROMATLAS_ROOT` can
override its Python toolkit location, but Cargo still needs the sibling layout.

The Pages builder also installs the local `scripts/browser-gamepads.mjs` adapter.
It accepts controllers without a browser standard mapping, preserves default
left-stick controls, and exposes extra buttons/axes for rebinding. Settings
capture reads every detected controller independently of player assignments and
waits for initially held controls to be released. Raw device layouts vary; use
Settings to map devices whose button ordering differs from the standard layout.
Run `node scripts/test-browser-gamepads.mjs` after building to check the adapter
and generated shell with simulated standard/raw controllers.

Both `run.py web` / `run.py web-build` and the Pages builder install the
versioned `web/index.html`, `web/shell.js` and `web/save-state.mjs` over the
toolkit-generated shell. They use the same local page installer, so Save State
and Restore State are available in local development and on Pages, including
with the older pinned toolkit revision. Restart an already running local web
server after page changes; its routes cache file contents at startup. The shared ROMAtlas template contains the same defaults.
After building, run the layout/focus/Reset check with Playwright installed:
`/root/romatlas/tools/web-tests/bin/python scripts/test-browser-page.py`.

## Browser save state

This port enables ROMAtlas's opt-in `snapshots` feature, available in published
revision `4f89589581fc7ea0173ce2c4d17fcbac1cd788f9`. The Pages workflow checks out that revision directly;
it does not modify toolkit source during CI. Local ROMAtlas checkouts need that
revision or later. The shared browser template keeps its original controls.

Save State stores one checkpoint in browser localStorage; another save replaces
that slot. Restore State reconstructs the saved game and leaves it paused at the
saved frame. Click Start to continue. The slot survives page reloads and Reset;
it belongs to the same site origin/browser profile, so clearing site data removes
it. This feature is specific to DuckTales and does not alter ROMAtlas templates
or the original cartridge's no-battery behavior.

The checkpoint serializes the current Rust simulation, suspended continuations,
pending display work, full audio/APU state, browser clocks and input mapper.
Immutable ROM resources are reattached from the current load. Restore does not
execute recorded inputs and does not get slower as the session grows. The slot
is limited to 4 MiB of JSON; there is no runtime-call or play-duration limit.

Snapshots include a checksum and validate the Wasm build, ROM, assets and audio
sample rate. Incompatible or corrupted data is rejected without replacing the
live game. Browser preferences stay current on restore. Storage errors leave the
prior slot intact. Old replay checkpoints are not snapshot-compatible; Save State
creates the new version-2 format in the same slot.

Verification with Playwright installed:
`/root/romatlas/tools/web-tests/bin/python scripts/test-browser-state.py`.

Browser toolbar shortcuts follow the button order: F1 starts/resumes, F2 pauses,
F3 resets, F4 saves the state slot, F5 restores it, and F6 opens/closes Settings. Shortcuts also work in
fullscreen. Inside Settings, Enter saves the form and Escape cancels; binding
capture takes precedence so a captured key does not trigger a toolbar action.
