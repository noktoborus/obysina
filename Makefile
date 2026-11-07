# The application identifier, which is what a desktop names the program by:
# the desktop entry, the icon and the `app_id` the window carries are one name.
ID = ru.styxheim.obusina

PREFIX ?= /usr/local
BINDIR = $(DESTDIR)$(PREFIX)/bin
APPDIR = $(DESTDIR)$(PREFIX)/share/applications
ICONDIR = $(DESTDIR)$(PREFIX)/share/icons/hicolor

.PHONY: build build-windows build-android apk apk-release aab install uninstall mockups version

# `make VERSION=x.y.z` sets the version of the program and tags it, and does
# nothing else (the target `version` at the end).
ifdef VERSION
.DEFAULT_GOAL := version
endif

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
# ABI itself. It needs ANDROID_HOME and ANDROID_NDK_HOME.
#
# `apk` is a release signed with the debug key DEBUG_KEY, made by keytool when
# it is missing: target/obusina.apk, installable by hand, not for a store.
# The key lives outside the repositories (android/README.md).
# `apk-release` is signed when OBUSINA_KEYSTORE names a keystore, else
# unsigned: target/obusina-release.apk.
DEBUG_KEY ?= $(abspath ../android-key/debug.keystore)
# The build tools of /opt/packages.sh, which the Android Gradle plugin uses.
BUILD_TOOLS = $(ANDROID_HOME)/build-tools/36.0.0

apk: $(DEBUG_KEY)
	cd android && env -u OBUSINA_KEYSTORE ./gradlew --no-daemon assembleRelease
	mkdir -p target
	$(BUILD_TOOLS)/zipalign -c -p 4 android/app/build/outputs/apk/release/app-release-unsigned.apk
	$(BUILD_TOOLS)/apksigner sign --ks $(DEBUG_KEY) --ks-pass pass:android \
		--ks-key-alias androiddebugkey --key-pass pass:android \
		--out target/obusina.apk android/app/build/outputs/apk/release/app-release-unsigned.apk
	rm -f target/obusina.apk.idsig
	$(BUILD_TOOLS)/apksigner verify target/obusina.apk

# The debug key: the one the Android Gradle plugin makes, with its passwords.
$(DEBUG_KEY):
	mkdir -p $(dir $(DEBUG_KEY))
	keytool -genkeypair -keystore $(DEBUG_KEY) -storepass android \
		-alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 \
		-validity 10000 -dname "CN=Android Debug,O=Android,C=US"

apk-release:
	cd android && ./gradlew --no-daemon assembleRelease
	mkdir -p target
	cp android/app/build/outputs/apk/release/app-release*.apk target/obusina-release.apk

aab:
	cd android && ./gradlew --no-daemon bundleRelease

install:
	@test -x target/release/obusina || { echo "run 'make build' first"; exit 1; }
	sudo install -Dm755 target/release/obusina $(BINDIR)/obusina
	sudo install -Dm644 assets/$(ID).desktop $(APPDIR)/$(ID).desktop
	sudo install -Dm644 assets/icon.svg $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true

# The mockups of experiments/mockups, built into the module `mockups` of the
# desk `obusina` of the server of /workspace/showcase on port 6080. The server
# is started if none answers, and lives on after the command. trunk and
# wasm-bindgen are the ones /opt/packages.sh installs: run after
# `. /opt/env.sh`.
mockups:
	cargo build --manifest-path ../showcase/Cargo.toml --bin showcase
	../showcase/target/debug/showcase up
	../showcase/target/debug/showcase build experiments/mockups --desk obusina --as mockups

uninstall:
	sudo rm -f $(BINDIR)/obusina
	sudo rm -f $(APPDIR)/$(ID).desktop
	sudo rm -f $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true

# The version of the program, `make VERSION=x.y.z`: `version` of
# `[workspace.package]` of Cargo.toml, which the android package reads too.
# Cargo.toml and Cargo.lock go in one commit, and the tag `vx.y.z` on it; the
# tag is not pushed. It needs a clean tree and a tag that is not there yet.
# 1.0.0 and on are refused while AGENTS.md still has the EXC of RULE
# pattern.version that holds only before 1.0.0.
version:
	@echo "$(VERSION)" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$$' \
		|| { echo "give VERSION=x.y.z"; exit 1; }
	@test -z "$$(git status --porcelain)" \
		|| { echo "the tree has changes: commit them first"; exit 1; }
	@! git rev-parse -q --verify "refs/tags/v$(VERSION)" >/dev/null \
		|| { echo "v$(VERSION) is a tag already"; exit 1; }
	@case "$(VERSION)" in 0.*) ;; *) ! grep -q 'EXC  before version 1.0.0' AGENTS.md \
		|| { echo "delete the EXC of RULE pattern.version first"; exit 1; };; esac
	sed -i '/^\[workspace.package\]/,/^\[/ s/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	grep -q '^version = "$(VERSION)"' Cargo.toml
	cargo update --workspace --offline
	git commit -q -m "Set the version to $(VERSION)" Cargo.toml Cargo.lock
	git tag -a "v$(VERSION)" -m "Obusina $(VERSION)"
	@echo "v$(VERSION): git push && git push origin v$(VERSION)"
