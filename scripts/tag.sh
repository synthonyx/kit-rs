#!/bin/bash
version=$(grep '^version' Cargo.toml | cut -d '=' -f 2- | xargs)
if [[ "$version" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  major=$(echo "$version" | cut -d '.' -f 1)
  minor=$(echo "$version" | cut -d '.' -f 2)
  patch=$(echo "$version" | cut -d '.' -f 3)
else
  echo "Error: Unable to parse semver from Cargo.toml." >&2
  exit 1
fi

version=$major.$minor.$patch
git tag -a "v$version" -m "Release version $version"
git push origin "v$version"
