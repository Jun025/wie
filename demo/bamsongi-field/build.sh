#!/usr/bin/env bash
# Builds out/bamsongi-field.jar — the recipe is shared: ../arcade-common/build-game.sh
exec "$(dirname "$0")/../arcade-common/build-game.sh" "$(dirname "$0")" bamsongi-field
