# The application identifier, which is what a desktop names the program by:
# the desktop entry, the icon and the `app_id` the window carries are one name.
ID = ru.styxheim.obusina

PREFIX ?= /usr/local
BINDIR = $(DESTDIR)$(PREFIX)/bin
APPDIR = $(DESTDIR)$(PREFIX)/share/applications
ICONDIR = $(DESTDIR)$(PREFIX)/share/icons/hicolor

.PHONY: build build-windows build-android apk apk-release aab install uninstall mockups

build:
	cargo build --release

# Windows, built with mingw: target/x86_64-pc-windows-gnu/release/obusina.exe.
build-windows:
	cargo build --release --target x86_64-pc-windows-gnu -p obusina

# The library android loads, built with the NDK of ANDROID_NDK_HOME:
# target/aarch64-linux-android/release/libobusina_android.so.
build-android:
	cargo ndk -t arm64-v8a build --release -p obusina-android

# The package of android, built by gradle, which builds the library of every
# ABI itself. It needs ANDROID_HOME and ANDROID_NDK_HOME. A release is signed
# when OBUSINA_KEYSTORE names a keystore (android/README.md), else unsigned.
apk:
	cd android && ./gradlew --no-daemon assembleDebug

apk-release:
	cd android && ./gradlew --no-daemon assembleRelease

aab:
	cd android && ./gradlew --no-daemon bundleRelease

install:
	@test -x target/release/obusina || { echo "run 'make build' first"; exit 1; }
	sudo install -Dm755 target/release/obusina $(BINDIR)/obusina
	sudo install -Dm644 assets/$(ID).desktop $(APPDIR)/$(ID).desktop
	sudo install -Dm644 assets/icon.svg $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true

# The mockups of experiments/mockups on port 6080, rebuilt and reloaded in
# the browser when a file of them changes. trunk and wasm-bindgen are the
# ones /opt/packages.sh installs: run after `. /opt/env.sh`.
mockups:
	cd experiments/mockups && trunk serve

uninstall:
	sudo rm -f $(BINDIR)/obusina
	sudo rm -f $(APPDIR)/$(ID).desktop
	sudo rm -f $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true
