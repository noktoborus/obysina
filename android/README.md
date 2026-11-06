# obusina-android

The library android loads: `libobusina_android.so`. `MainActivity`, the game
activity of `app/src/main/java`, calls `android_main`, which calls
`obusina::android`.

## Scope

- The `cdylib` of the program for android, nothing else.
- On any other platform it is empty, so `cargo build --workspace` builds it
  everywhere and it costs nothing.

- `MainActivity.pickPicture` opens the choice of a document of the system
  for a picture; its answer comes back through the native method
  `picturePicked`, exported here, to `obusina::android_picked`
  (`docs/backdrop.md`).

## Boundaries

No code of its own beyond the entries. What the program does is the root
package.

## Errors

None: `obusina::android` logs what fails.

## The package

`android/` is also a gradle project. `app/build.gradle.kts` runs cargo-ndk
for every ABI of the package before gradle packs the libraries:

```text
gradle assemble<Variant> ──► cargoDebug | cargoRelease
                              cargo ndk --platform <minSdk> -t <abi>… build
                              ──► app/build/rust/<variant>/<abi>/*.so
                         ──► merge<Variant>JniLibFolders ──► APK, AAB
```

| Command | Result |
| --- | --- |
| `make apk` | `target/obusina.apk` of the workspace, a release signed with the debug key |
| `make apk-release` | `target/obusina-release.apk` of the workspace, signed or not, see below |
| `make aab` | `app/build/outputs/bundle/release/app-release.aab` |

`make apk` signs with the debug key `DEBUG_KEY` of the Makefile,
`/workspace/android-key/debug.keystore`, beside the repositories so it
outlives the container and no repository holds it; `keytool` makes it when
it is missing. Its passwords are those of the debug key of the Android
Gradle plugin. A package signed with it installs by hand; one signed with
another key does not install over it.

`make apk-release` is signed when `OBUSINA_KEYSTORE` names a keystore;
`OBUSINA_KEYSTORE_PASSWORD`, `OBUSINA_KEY_ALIAS` and `OBUSINA_KEY_PASSWORD`
give the rest. Without it the release is unsigned. No key lives in the
repository.

The versions of the SDK, the NDK and the libraries are in
`app/build.gradle.kts` and `build.gradle.kts`.
