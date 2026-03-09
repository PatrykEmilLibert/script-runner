# macOS Build Instructions

## Prerequisites
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install Node.js: Download from https://nodejs.org/
3. Install Xcode Command Line Tools: `xcode-select --install`
4. For distribution builds: Apple Developer account (for codesigning + notarization)

## Local Build (unsigned, for development)
```bash
cd script-runner
npm install
chmod +x build.sh
./build.sh
```

Release builds create Python runtime in `src-tauri/python` and bundle it into app resources.
Deployment target is pinned to macOS 11.0 for better compatibility with older supported systems.

Unsigned builds can run locally, but downloaded artifacts may be blocked by Gatekeeper.

## CI/Release Build (signed + notarized)
Configure GitHub Actions secrets in repository settings:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (optional if key has no passphrase)
- `TAURI_UPDATER_PUBLIC_KEY`
- `SR_SCRIPTS_PUSH_TOKEN_B64` (optional, base64-encoded GitHub PAT for technical bot account to push scripts without user login)
- `APPLE_CERTIFICATE` (base64-encoded `.p12` certificate)
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY` (e.g. `Developer ID Application: ...`)
- `APPLE_ID` (Apple account email)
- `APPLE_PASSWORD` (app-specific password)
- `APPLE_TEAM_ID`

When all secrets are present, the macOS workflow passes them to Tauri build and produces signed/notarized artifacts plus updater metadata (`.sig`, `latest.json`).

## Output
The macOS app bundle will be created at:
- `src-tauri/target/release/bundle/dmg/ScriptRunner_*.dmg`
- `src-tauri/target/release/bundle/macos/ScriptRunner.app`

Updater artifacts (release config only):
- `src-tauri/target/release/bundle/macos/*.app.tar.gz`
- `src-tauri/target/release/bundle/macos/*.app.tar.gz.sig`

## Running
Double-click the `.dmg` file to mount it, then drag ScriptRunner to Applications folder.

For unsigned builds, the CI release also includes `START_MAC.command`.
After moving app to Applications, run `START_MAC.command` once to remove quarantine and launch ScriptRunner.
If needed, the script now asks for admin password and also adds app to Gatekeeper exceptions automatically.
If double-click does not start it, run: `bash START_MAC.command` in Terminal.

## Troubleshooting: "ScriptRunner is damaged and can't be opened"
This usually means the app is unsigned/not notarized or quarantined by macOS.

1. Preferred fix (production): build with signing + notarization secrets configured.
2. Temporary workaround (internal testing only): remove quarantine attribute:

```bash
xattr -dr com.apple.quarantine /Applications/ScriptRunner.app
```

Alternative for non-technical users: run `START_MAC.command` from release assets.

3. Verify signature/trust status:

```bash
codesign --verify --deep --strict --verbose=2 /Applications/ScriptRunner.app
spctl -a -vv /Applications/ScriptRunner.app
```

## Admin Access Setup
Admin features now require GitHub admin login (PAT + `admins.json` in `script-runner-config`).
See `GITHUB_AUTH_GUIDE.md` for setup details.
