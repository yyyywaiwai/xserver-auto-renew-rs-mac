#!/usr/bin/env bash
set -euo pipefail

repo="h-sumiya/xserver-auto-renew-rs"
version="${VERSION:-latest}"

echo "xrenew installed! まず 'xrenew login' \n次に 'xrenew {定期実行を有効化}' を実行してください。"
