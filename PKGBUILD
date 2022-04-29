pkgname=monitor-ipv6
pkgver=1.2.0
pkgrel=1
pkgdesc='Monitor interface for ipv6 address'
arch=('x86_64' 'armv7h' 'aarch64')
depends=()
makedepends=(rust)

build() {
  cargo +nightly build --release --locked
  strip ../target/release/monitor-ipv6
}

package()
{
  cd $pkgdir/../..
  install -Dm 755 "target/release/monitor-ipv6" -t "${pkgdir}/usr/bin"
  install -Dm 644 "monitor-ipv6.service" -t "${pkgdir}/usr/lib/systemd/system"
}
