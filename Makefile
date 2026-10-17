# The application identifier, which is what a desktop names the program by:
# the desktop entry, the icon and the `app_id` the window carries are one name.
ID = ru.styxheim.obusina

PREFIX ?= /usr/local
BINDIR = $(DESTDIR)$(PREFIX)/bin
APPDIR = $(DESTDIR)$(PREFIX)/share/applications
ICONDIR = $(DESTDIR)$(PREFIX)/share/icons/hicolor

.PHONY: build install uninstall

build:
	cargo build --release

install:
	@test -x target/release/obusina || { echo "run 'make build' first"; exit 1; }
	sudo install -Dm755 target/release/obusina $(BINDIR)/obusina
	sudo install -Dm644 assets/$(ID).desktop $(APPDIR)/$(ID).desktop
	sudo install -Dm644 assets/icon.svg $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true

uninstall:
	sudo rm -f $(BINDIR)/obusina
	sudo rm -f $(APPDIR)/$(ID).desktop
	sudo rm -f $(ICONDIR)/scalable/apps/$(ID).svg
	sudo update-desktop-database $(APPDIR) 2>/dev/null || true
