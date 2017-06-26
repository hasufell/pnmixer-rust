INSTALL = install
INSTALL_DIR = $(INSTALL) -d
INSTALL_BIN = $(INSTALL) -m 755
INSTALL_DATA = $(INSTALL) -m 644


PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
SHAREDIR=$(PREFIX)/share
DATADIR=$(SHAREDIR)/pnmixer
PIXMAPSDIR=$(DATADIR)/pixmaps
ICONSDIR=$(SHAREDIR)/icons/hicolor/128x128/apps
DESKTOPDIR=$(SHAREDIR)/applications


CARGO ?= cargo
CARGO_ARGS ?= 
CARGO_BUILD_ARGS ?= --release
CARGO_BUILD ?= $(CARGO) $(CARGO_ARGS) build $(CARGO_BUILD_ARGS)
CARGO_INSTALL_ARGS ?= --root="$(DESTDIR)/$(PREFIX)"
CARGO_INSTALL ?= $(CARGO) $(CARGO_ARGS) install $(CARGO_INSTALL_ARGS)



pnmixer-rs: Cargo.toml
	PIXMAPSDIR=$(PIXMAPSDIR) $(CARGO_BUILD)


install: install-data
	$(INSTALL_DIR) "$(DESTDIR)/$(BINDIR)"
	$(INSTALL_BIN) target/release/pnmixer "$(DESTDIR)/$(BINDIR)/pnmixer"


install-data: install-pixmaps install-icons install-desktop


install-pixmaps:
	$(INSTALL_DIR) "$(DESTDIR)/$(PIXMAPSDIR)"
	$(INSTALL_DATA) data/pixmaps/pnmixer-about.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-about.png"
	$(INSTALL_DATA) data/pixmaps/pnmixer-high.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-high.png"
	$(INSTALL_DATA) data/pixmaps/pnmixer-low.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-low.png"
	$(INSTALL_DATA) data/pixmaps/pnmixer-medium.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-medium.png"
	$(INSTALL_DATA) data/pixmaps/pnmixer-muted.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-muted.png"
	$(INSTALL_DATA) data/pixmaps/pnmixer-off.png "$(DESTDIR)/$(PIXMAPSDIR)/pnmixer-off.png"


install-icons:
	$(INSTALL_DIR) "$(DESTDIR)/$(ICONSDIR)"
	$(INSTALL_DATA) data/icons/pnmixer.png "$(DESTDIR)/$(ICONSDIR)/pnmixer.png"


install-desktop:
	$(INSTALL_DIR) "$(DESTDIR)/$(DESKTOPDIR)"
	$(INSTALL_DATA) data/desktop/pnmixer.desktop "$(DESTDIR)/$(DESKTOPDIR)/pnmixer.desktop"



.PHONY: pnmixer-rs install install-data install-pixmaps install-icons install-desktop
