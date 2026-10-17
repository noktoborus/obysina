# obusina-android

The library android loads: `libobusina_android.so`. The game activity calls
`android_main`, which calls `obusina::android`.

## Scope

- The `cdylib` of the program for android, nothing else.
- On any other platform it is empty, so `cargo build --workspace` builds it
  everywhere and it costs nothing.

## Boundaries

No code of its own beyond the entry. What the program does is the root
package.

## Errors

None: `obusina::android` logs what fails.

## Building

```sh
cargo ndk -t arm64-v8a build -p obusina-android
```
