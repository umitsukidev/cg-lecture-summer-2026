#!/bin/sh

# Build every nannou binary in the Cargo workspace independently.
# Every Wasm file is Brotli-compressed in place for Cloudflare Static Assets.
# worker/index.js serves these files with manual Content-Encoding handling.
# A failed app is reported, but is not copied into the final dist directory.

set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
source_path_root=$(CDPATH= cd -- "${HOME:?HOME is required}" && pwd)
rust_path_remap_flag="--remap-path-prefix=$source_path_root=source"
rustflags_separator=$(printf '\037')
dist_dir="$repo_root/dist"
cloudflare_max_asset_size=26214400
metadata_file=$(mktemp "${TMPDIR:-/tmp}/nannou-dist-metadata.XXXXXX")
packages_file=$(mktemp "${TMPDIR:-/tmp}/nannou-dist-packages.XXXXXX")
success_file=$(mktemp "${TMPDIR:-/tmp}/nannou-dist-success.XXXXXX")
failure_file=$(mktemp "${TMPDIR:-/tmp}/nannou-dist-failure.XXXXXX")
stage_dir=$(mktemp -d "$repo_root/.dist-build.XXXXXX")
generated_index=
old_dist=

cleanup() {
  if [ -n "$generated_index" ]; then
    rm -f -- "$generated_index"
  fi
  rm -f -- "$metadata_file" "$packages_file" "$success_file" "$failure_file"
  if [ -n "$stage_dir" ]; then
    rm -rf -- "$stage_dir"
  fi
  if [ -n "$old_dist" ] && [ -e "$old_dist" ]; then
    if [ -e "$dist_dir" ]; then
      rm -rf -- "$old_dist"
    else
      mv -- "$old_dist" "$dist_dir"
    fi
  fi
}

trap cleanup EXIT HUP INT TERM

for command_name in brotli cargo trunk python3; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "error: required command not found: $command_name" >&2
    exit 1
  fi
done

prepare_static_assets() {
  prepare_output_dir=$1
  prepare_file_list=$2

  find "$prepare_output_dir" -type f -name '*.wasm' -print >"$prepare_file_list"
  if [ ! -s "$prepare_file_list" ]; then
    echo "error: Trunk output contains no Wasm file" >&2
    return 1
  fi

  while IFS= read -r prepare_wasm_file; do
    prepare_compressed_file="$prepare_wasm_file.br"
    prepare_original_size=$(wc -c <"$prepare_wasm_file" | tr -d ' ')

    if LC_ALL=C grep -a -q -F "$source_path_root" "$prepare_wasm_file"; then
      echo "error: Wasm contains a local absolute source path: $prepare_wasm_file" >&2
      return 1
    fi

    if ! brotli \
      --quality=9 \
      --force \
      --output="$prepare_compressed_file" \
      "$prepare_wasm_file"
    then
      echo "error: Brotli compression failed: $prepare_wasm_file" >&2
      return 1
    fi

    if ! brotli --test "$prepare_compressed_file"; then
      echo "error: Brotli verification failed: $prepare_compressed_file" >&2
      return 1
    fi

    prepare_compressed_size=$(wc -c <"$prepare_compressed_file" | tr -d ' ')
    if [ "$prepare_compressed_size" -ge "$prepare_original_size" ]; then
      echo "error: Brotli did not reduce the Wasm file: $prepare_wasm_file" >&2
      return 1
    fi

    mv -- "$prepare_compressed_file" "$prepare_wasm_file"
    echo "BR    ${prepare_original_size} -> ${prepare_compressed_size} bytes"
  done <"$prepare_file_list"

  find "$prepare_output_dir" -type f -print >"$prepare_file_list"
  while IFS= read -r prepare_asset_file; do
    prepare_asset_size=$(wc -c <"$prepare_asset_file" | tr -d ' ')
    if [ "$prepare_asset_size" -gt "$cloudflare_max_asset_size" ]; then
      echo "error: asset still exceeds Cloudflare's 25 MiB limit: $prepare_asset_file" >&2
      return 1
    fi
  done <"$prepare_file_list"
}

echo "Discovering nannou apps..."
if ! cargo metadata \
  --manifest-path "$repo_root/Cargo.toml" \
  --format-version 1 \
  --no-deps >"$metadata_file"
then
  echo "error: cargo metadata failed" >&2
  exit 1
fi

python3 - "$metadata_file" >"$packages_file" <<'PY'
import json
import pathlib
import sys

metadata_path = pathlib.Path(sys.argv[1])
metadata = json.loads(metadata_path.read_text(encoding="utf-8"))

for package in metadata["packages"]:
    if not any(dependency["name"] == "nannou" for dependency in package["dependencies"]):
        continue

    binaries = [
        target["name"]
        for target in package["targets"]
        if "bin" in target["kind"]
    ]
    for binary in binaries:
        print(
            package["name"],
            binary,
            package["manifest_path"],
            len(binaries),
            sep="\t",
        )
PY

app_count=$(wc -l <"$packages_file" | tr -d ' ')
if [ "$app_count" -eq 0 ]; then
  echo "error: no nannou binary packages found in the workspace" >&2
  exit 1
fi

echo "Found $app_count nannou app(s)."
echo

# Trunk 0.21 expects NO_COLOR to contain "true" or "false", while many
# terminals conventionally set it to "1". Color is not relevant to the build.
unset NO_COLOR

while IFS="$(printf '\t')" read -r package_name binary_name manifest_path binary_count; do
  package_dir=${manifest_path%/Cargo.toml}

  case "$package_dir" in
    "$repo_root")
      package_route=$package_name
      ;;
    "$repo_root"/*)
      package_route=${package_dir#"$repo_root"/}
      ;;
    *)
      echo "SKIP  $package_name ($binary_name): package is outside the repository" >&2
      printf '%s\t%s\t%s\n' "$package_name" "$binary_name" "outside repository" >>"$failure_file"
      continue
      ;;
  esac

  if [ "$binary_count" -gt 1 ]; then
    app_route="$package_route/$binary_name"
    input_html=
  else
    app_route=$package_route
    if [ -f "$package_dir/index.html" ]; then
      input_html="$package_dir/index.html"
    else
      input_html=
    fi
  fi

  if [ -z "$input_html" ]; then
    generated_index="$package_dir/.trunk-dist-index.html"
    if [ -e "$generated_index" ]; then
      echo "SKIP  $package_name ($binary_name): reserved file already exists: $generated_index" >&2
      printf '%s\t%s\t%s\n' "$package_name" "$binary_name" "reserved input file exists" >>"$failure_file"
      generated_index=
      continue
    fi

    cat >"$generated_index" <<EOF
<!doctype html>
<html lang="ja">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>$package_name — $binary_name</title>
    <link data-trunk rel="rust" href="Cargo.toml" data-bin="$binary_name" data-wasm-opt="z" />
    <style>
      html, body {
        width: 100%;
        height: 100%;
        margin: 0;
        overflow: hidden;
        background: #111;
      }
      body {
        display: grid;
        place-items: center;
      }
      canvas {
        max-width: 100%;
        max-height: 100%;
      }
    </style>
  </head>
  <body></body>
</html>
EOF
    input_html=$generated_index
  fi

  build_dir=$(mktemp -d "${TMPDIR:-/tmp}/nannou-dist-app.XXXXXX")
  build_output="$build_dir/output"

  echo "BUILD $package_name ($binary_name)"
  if (
    if [ -n "${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
      export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}${rustflags_separator}${rust_path_remap_flag}"
    else
      export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }${rust_path_remap_flag}"
    fi

    cd -- "$package_dir" &&
      trunk build \
        "$input_html" \
        --dist "$build_output" \
        --public-url "./" \
        --features "tracing/release_max_level_off,log/release_max_level_off" \
        --cargo-profile wasm-release
  )
  then
    if prepare_static_assets "$build_output" "$build_dir/asset-files"; then
      final_app_dir="$stage_dir/$app_route"
      mkdir -p -- "$final_app_dir"
      cp -R -- "$build_output/." "$final_app_dir/"
      printf '%s\t%s\t%s\n' "$package_name" "$binary_name" "$app_route" >>"$success_file"
      echo "OK    $app_route/index.html"
    else
      printf '%s\t%s\t%s\n' "$package_name" "$binary_name" "asset preparation failed" >>"$failure_file"
      echo "FAIL  $package_name ($binary_name): asset preparation failed" >&2
    fi
  else
    printf '%s\t%s\t%s\n' "$package_name" "$binary_name" "trunk build failed" >>"$failure_file"
    echo "FAIL  $package_name ($binary_name)" >&2
  fi

  rm -rf -- "$build_dir"
  if [ -n "$generated_index" ]; then
    rm -f -- "$generated_index"
    generated_index=
  fi
  echo
done <"$packages_file"

success_count=$(wc -l <"$success_file" | tr -d ' ')
failure_count=$(wc -l <"$failure_file" | tr -d ' ')

cat >"$stage_dir/index.html" <<'EOF'
<!doctype html>
<html lang="ja">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>nannou apps</title>
    <style>
      :root {
        color-scheme: dark;
        font-family: ui-sans-serif, system-ui, sans-serif;
        background: #101114;
        color: #f4f4f5;
      }
      body {
        width: min(56rem, calc(100% - 2rem));
        margin: 4rem auto;
      }
      h1 {
        margin-bottom: 0.5rem;
      }
      p {
        color: #a1a1aa;
      }
      ul {
        display: grid;
        gap: 0.75rem;
        padding: 0;
        list-style: none;
      }
      a {
        display: block;
        padding: 1rem 1.25rem;
        border: 1px solid #303038;
        border-radius: 0.75rem;
        background: #18181d;
        color: #e4e4e7;
        text-decoration: none;
      }
      a:hover {
        border-color: #71717a;
        background: #202027;
      }
      code {
        color: #a1a1aa;
      }
    </style>
  </head>
  <body>
    <h1>nannou apps</h1>
EOF

printf '    <p>%s app(s) built successfully.</p>\n' "$success_count" >>"$stage_dir/index.html"
printf '    <ul>\n' >>"$stage_dir/index.html"
while IFS="$(printf '\t')" read -r package_name binary_name app_route; do
  printf '      <li><a href="%s/index.html">%s <code>%s</code></a></li>\n' \
    "$app_route" "$package_name" "$app_route" >>"$stage_dir/index.html"
done <"$success_file"
cat >>"$stage_dir/index.html" <<'EOF'
    </ul>
  </body>
</html>
EOF

if [ -e "$dist_dir" ]; then
  old_dist="$repo_root/.dist-build.old.$$"
  mv -- "$dist_dir" "$old_dist"
fi

if mv -- "$stage_dir" "$dist_dir"; then
  stage_dir=
  if [ -n "$old_dist" ]; then
    rm -rf -- "$old_dist"
  fi
else
  echo "error: could not install the completed dist directory" >&2
  if [ -n "$old_dist" ] && [ ! -e "$dist_dir" ]; then
    mv -- "$old_dist" "$dist_dir"
  fi
  exit 1
fi

echo "Built $success_count app(s); $failure_count app(s) failed."
echo "Output: $dist_dir"

if [ "$failure_count" -gt 0 ]; then
  echo
  echo "Failed app(s):"
  while IFS="$(printf '\t')" read -r package_name binary_name reason; do
    echo "  - $package_name ($binary_name): $reason"
  done <"$failure_file"
fi

if [ "$success_count" -eq 0 ]; then
  exit 1
fi
