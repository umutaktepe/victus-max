# Maintainer: Umut Aktepe <umutaktepe>

pkgname=victus-max-git
_pkgname=victus-max
pkgver=2.1.2
pkgrel=1
pkgdesc="Advanced HP Victus and OMEN laptop manager for Linux with Better Auto cooling, Fan, RGB, and MUX control"
arch=('x86_64')
url="https://github.com/umutaktepe/victus-max"
license=('GPL')
depends=('dkms' 'polkit' 'gtk4' 'libadwaita' 'gtk4-layer-shell')
makedepends=('git' 'gcc' 'make' 'pkg-config' 'rust')
provides=('victus-max' 'omen-space')
conflicts=('victus-max' 'omen-space' 'hp-laptop-manager' 'omenctl')
source=('git+https://github.com/umutaktepe/victus-max.git')
sha256sums=('SKIP')

pkgver() {
  cd "$srcdir/${pkgname%-git}"
  git describe --long --tags | sed 's/\([^-]*-\)g/r\1/;s/-/./g' | sed 's/^v//'
}

build() {
  cd "$srcdir/${pkgname%-git}"
  cargo build --release --locked
}

package() {
  cd "$srcdir/${pkgname%-git}"

  # Install directories
  mkdir -p "$pkgdir/usr/libexec/victus-max"
  mkdir -p "$pkgdir/usr/libexec/omen-space"
  mkdir -p "$pkgdir/etc/victus-max"
  mkdir -p "$pkgdir/etc/omen-space"
  mkdir -p "$pkgdir/etc/dbus-1/system.d"
  mkdir -p "$pkgdir/usr/lib/systemd/system"
  mkdir -p "$pkgdir/usr/lib/sysusers.d"
  mkdir -p "$pkgdir/usr/lib/udev/rules.d"
  mkdir -p "$pkgdir/usr/bin"
  mkdir -p "$pkgdir/usr/share/applications"
  mkdir -p "$pkgdir/usr/share/dbus-1/services"
  mkdir -p "$pkgdir/usr/share/pixmaps"
  mkdir -p "$pkgdir/usr/share/icons/hicolor/512x512/apps"
  mkdir -p "$pkgdir/usr/share/victus-max/assets"
  mkdir -p "$pkgdir/usr/share/omen-space/assets"
  mkdir -p "$pkgdir/etc/xdg/autostart"

  # Binaries
  cp target/release/victus-max-daemon "$pkgdir/usr/libexec/victus-max/"
  ln -sf /usr/libexec/victus-max/victus-max-daemon "$pkgdir/usr/libexec/omen-space/omen-space-daemon"

  cp target/release/victus-max-cli "$pkgdir/usr/bin/"
  ln -sf /usr/bin/victus-max-cli "$pkgdir/usr/bin/omen-cli"

  cp target/release/victus-max-tray "$pkgdir/usr/bin/"
  ln -sf /usr/bin/victus-max-tray "$pkgdir/usr/bin/omen-tray"

  cp target/release/victus-max "$pkgdir/usr/bin/"
  ln -sf /usr/bin/victus-max "$pkgdir/usr/bin/victus-max-gui"
  ln -sf /usr/bin/victus-max "$pkgdir/usr/bin/omen-gui"

  if [ -f target/release/victus-max-overlay ]; then
    cp target/release/victus-max-overlay "$pkgdir/usr/bin/"
    ln -sf /usr/bin/victus-max-overlay "$pkgdir/usr/bin/omen-overlay"
  fi

  # System configuration files
  cp data/org.hp.omen.conf "$pkgdir/etc/dbus-1/system.d/"
  cp data/victus-max-daemon.service "$pkgdir/usr/lib/systemd/system/"
  ln -sf victus-max-daemon.service "$pkgdir/usr/lib/systemd/system/omen-space-daemon.service"
  cp data/sysusers.d/omen-space.conf "$pkgdir/usr/lib/sysusers.d/"
  cp data/99-omen-space.rules "$pkgdir/usr/lib/udev/rules.d/"

  # Desktop integration and assets
  cp data/org.hp.VictusMax.desktop "$pkgdir/usr/share/applications/"
  cp data/org.hp.OmenSpace.desktop "$pkgdir/usr/share/applications/"
  cp data/org.hp.VictusMax.service "$pkgdir/usr/share/dbus-1/services/"
  cp data/org.hp.OmenSpace.service "$pkgdir/usr/share/dbus-1/services/"
  cp src/victus-max-gui/assets/victus-max.png "$pkgdir/usr/share/pixmaps/victus-max.png"
  cp src/victus-max-gui/assets/victus-max.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/victus-max.png"
  cp src/victus-max-gui/assets/omenspace.png "$pkgdir/usr/share/pixmaps/omenspace.png"
  cp src/victus-max-gui/assets/omenspace.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/omenspace.png"
  cp -r src/victus-max-gui/assets/* "$pkgdir/usr/share/victus-max/assets/"
  cp -r src/victus-max-gui/assets/* "$pkgdir/usr/share/omen-space/assets/"

  # Autostart tray
  cat <<EOF > "$pkgdir/etc/xdg/autostart/victus-max-tray.desktop"
[Desktop Entry]
Name=Victus Max Tray
Comment=Victus Max System Tray Icon
Exec=/usr/bin/victus-max-tray
Icon=victus-max
Terminal=false
Type=Application
Categories=Utility;
EOF

  # DKMS Driver
  _dkms_dir="$pkgdir/usr/src/hp-omen-extra-${pkgver}"
  mkdir -p "$_dkms_dir"
  cp driver/hp-wmi.c "$_dkms_dir/"
  cp driver/hp-omen-extra.c "$_dkms_dir/"
  cp driver/Makefile "$_dkms_dir/"
  cp driver/dkms.conf "$_dkms_dir/"

  # Set version in dkms.conf
  sed -i "s/PACKAGE_VERSION=.*/PACKAGE_VERSION=\"${pkgver}\"/" "$_dkms_dir/dkms.conf"
}
