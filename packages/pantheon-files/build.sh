#!/usr/bin/env bash
# Build and install pantheon-files from the Arch package with finder-clicks.patch
# applied (see the patch header). Run again after each pantheon-files update:
#   make files-patch
# Fetches the packaging repo as a tarball (no git), patches the PKGBUILD, runs makepkg -si.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
build_dir="${XDG_CACHE_HOME:-$HOME/.cache}/dotfiles-build/pantheon-files"
tarball="https://gitlab.archlinux.org/archlinux/packaging/packages/pantheon-files/-/archive/main/pantheon-files-main.tar.gz"

rm -rf "$build_dir"
mkdir -p "$build_dir"
curl -sSL "$tarball" | tar -xz --strip-components=1 -C "$build_dir"
cp "$here/finder-clicks.patch" "$build_dir/"

cd "$build_dir"
# Add the patch to the sources and apply it before the build.
python3 - <<'PY'
import pathlib, re
pkgbuild = pathlib.Path("PKGBUILD")
text = pkgbuild.read_text()
text = re.sub(r"^source=\(", "source=(finder-clicks.patch\n        ", text, count=1, flags=re.M)
text = re.sub(r"^(sha256sums|b2sums)=\(", r"\1=(SKIP\n        ", text, count=1, flags=re.M)
if "prepare()" in text:
    text = text.replace("prepare() {\n", "prepare() {\n  patch -d $pkgname -p1 < finder-clicks.patch\n", 1)
else:
    text = text.replace("\nbuild() {", "\nprepare() {\n  patch -d $pkgname -p1 < finder-clicks.patch\n}\n\nbuild() {", 1)
# No -debug package: makepkg -i would install it alongside.
text += "\noptions=(!debug)\n"
pkgbuild.write_text(text)
PY
grep -n "finder-clicks\|^pkgrel\|^prepare" PKGBUILD
# Same version string as the repo package, so plain -i is needed: --needed would skip it.
makepkg -si
