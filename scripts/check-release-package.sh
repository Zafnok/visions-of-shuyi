#!/usr/bin/env bash
# Checks a package `cargo shuyi export <target> --release` wrote (ticket
# 0910), as release.yml runs it: the final package's files against what
# shuyi.ron ships, whether the bought art is in the build, then by target:
#
# - windows: the icon and version info are in the program, which imports no
#   MinGW runtime DLL and has no debug sections, and the debug file has them;
# - linux: the same sections, and no glibc version above the sniper SDK's;
# - macos: the `.app` is signed ad hoc and has both slices;
# - web: the page's scripts are named in NOTICES.txt.
#
# Usage: bash scripts/check-release-package.sh <target> <export folder> [private-assets]
# From the repository's root, whose `music/`, `voice/` and license folders
# are what the package must hold.

set -euo pipefail
export LC_ALL=C

target=$1
out=$2
art=${3:-}
bin=visions-of-shuyi
report="$out/export-report.json"

fail() { echo "check: $*"; exit 1; }
# A string field of the report, which serde_json writes one to a line.
field() { sed -n "s/^  \"$1\": \"\(.*\)\",\{0,1\}\r\{0,1\}$/\1/p" "$report" | head -n 1; }

folder=$(field folder)
package=$(field package)
echo "check: $target $package"

# Every file of a folder of the repository, as the package names it.
tree() { (cd "$1" && find . -type f | sed "s|^\./|$2/|"); }

# What ships beside the program; folders under the prefix a target gives them.
beside() {
  local folders=$1
  printf '%s\n' README.txt LICENSE THIRD_PARTY_ASSETS.md THIRD_PARTY_LICENSES.html \
    Terminus-LICENSE.txt NOTICES.txt
  tree assets/audio/licenses "${folders}audio-licenses"
  tree music "${folders}music"
  if [ -d voice ]; then tree voice "${folders}voice"; fi
}

case "$target" in
  windows) expected=$({ beside ""; printf '%s\n' "$bin.exe" HOW-TO-OPEN.txt; } | sed "s|^|$folder/|") ;;
  linux) expected=$({ beside ""; echo "$bin"; } | sed "s|^|$folder/|") ;;
  macos)
    contents="$bin.app/Contents"
    expected=$({
      beside "$contents/Resources/"
      printf '%s\n' HOW-TO-OPEN.txt "$contents/Info.plist" \
        "$contents/_CodeSignature/CodeResources" "$contents/MacOS/$bin"
    } | sed "s|^|$folder/|")
    ;;
  web)
    expected=$({
      beside ""
      printf '%s\n' index.html "$bin.wasm" mq_js_bundle.js sapp_jsutils.js quad-storage.js gamepad.js
    })
    ;;
  *) fail "no such target: $target" ;;
esac

py=$(command -v python3 || command -v python || true)
case "$package" in
  *.zip) listed=$("$py" -c "import zipfile,sys; print('\n'.join(n for n in zipfile.ZipFile(sys.argv[1]).namelist() if not n.endswith('/')))" "$out/$package") ;;
  *.tar.gz) listed=$(tar tzf "$out/$package" | grep -v '/$') ;;
  *) listed=$(cd "$out" && find "$package" -type f) ;;
esac
listed=$(printf '%s\n' "$listed" | tr -d '\r' | sort)
expected=$(printf '%s\n' "$expected" | sort)
if [ "$listed" != "$expected" ]; then
  echo "check: the package's files differ (< expected, > in the package):"
  diff <(printf '%s\n' "$expected") <(printf '%s\n' "$listed") || true
  exit 1
fi
echo "check: $(printf '%s\n' "$listed" | wc -l | tr -d ' ') files, as expected"

# The build's features, as the exposure report read them.
features=$(sed -n '/"features": \[/,/\]/p' "$out/exposure-report.json")
if [ "$art" = private-assets ]; then
  case "$features" in *private-assets*) ;; *) fail "the bought art isn't in this build" ;; esac
else
  case "$features" in *private-assets*) fail "the bought art is in a build that said it wasn't" ;; esac
fi

# No debug sections in the program; the debug file has them at the same place.
sections() {
  local program=$1 debug="$out/$folder.debug" left a b
  # `.debug_gdb_scripts` is data the program loads, which `--strip-debug` keeps.
  left=$(objdump -h "$program" | awk '$2 ~ /^\.debug_/ && $2 != ".debug_gdb_scripts" { print $2 }')
  [ -z "$left" ] || fail "the program has debug sections: $left"
  objdump -h "$debug" | grep -q ' \.debug_line' || fail "$debug has no .debug_line"
  a=$(objdump -h "$program" | awk '$2 == ".text" { print $4 }')
  b=$(objdump -h "$debug" | awk '$2 == ".text" { print $4 }')
  [ -n "$a" ] && [ "$a" = "$b" ] || fail ".text at $a, in the debug file at $b"
}

case "$target" in
  windows)
    program="$out/$folder/$bin.exe"
    sections "$program"
    objdump -h "$program" | grep -q ' \.rsrc' || fail "no icon or version info in the program"
    dlls=$(objdump -p "$program" | awk '/DLL Name:/ { print $3 }' | sort -fu)
    echo "check: imports $(echo "$dlls" | tr '\n' ' ')"
    if echo "$dlls" | grep -qiE '^(libgcc|libwinpthread|libstdc\+\+)'; then
      fail "the program needs a MinGW runtime DLL"
    fi
    ;;
  linux)
    program="$out/$folder/$bin"
    sections "$program"
    tar tvzf "$out/$package" | grep -q "^-rwx.* $folder/$bin\$" || fail "the program isn't executable"
    floor=2.31
    newest=$(objdump -T "$program" | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -Vu | tail -n 1)
    echo "check: the newest glibc it asks for is $newest (the floor: $floor)"
    [ "$(printf '%s\n%s\n' "$newest" "$floor" | sort -V | tail -n 1)" = "$floor" ] ||
      fail "GLIBC_$newest is above the floor"
    ;;
  macos)
    app="$out/$folder/$bin.app"
    codesign --verify --strict --verbose=2 "$app"
    codesign -dv "$app" 2>&1 | grep -q 'Signature=adhoc' || fail "not signed ad hoc"
    archs=$(lipo -archs "$app/Contents/MacOS/$bin")
    echo "check: slices $archs"
    case " $archs " in *" arm64 "*) ;; *) fail "no arm64 slice" ;; esac
    case " $archs " in *" x86_64 "*) ;; *) fail "no x86_64 slice" ;; esac
    ;;
  web)
    for script in macroquad sapp-jsutils quad-storage; do
      grep -q "$script" "$out/$folder/NOTICES.txt" || fail "NOTICES.txt doesn't name $script"
    done
    ;;
esac
echo "check: $target OK"
