#!/usr/bin/env bash

# 1. Find all Cargo.toml files (skipping target dirs) and store them in an array.
# Using mapfile ensures `find` finishes its job completely before any cleaning starts.
mapfile -d $'\0' toml_files < <(find . -type d -name target -prune -o -type f -name Cargo.toml -print0)

# 2. Loop through our saved list and run cargo clean on each one.
for file in "${toml_files[@]}"; do
  # Just a quick check to make sure the file isn't empty/null
  if [[ -n "$file" ]]; then
    cargo +stable clean --manifest-path "$file"
  fi
done
