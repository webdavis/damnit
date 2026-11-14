#!/bin/sh
beside_this=$(dirname "$0")
DAM_TODOIST_BASE_URL=$(cat "$beside_this/base-url") exec "$(cat "$beside_this/helper")" "$@"
