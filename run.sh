#!/usr/bin/env bash

set -euo pipefail

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json

exec "$project_dir/target/debug/aetheris"
