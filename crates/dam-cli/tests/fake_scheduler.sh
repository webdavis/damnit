#!/bin/sh
here=${0%/*}
printf '%s %s\n' "${0##*/}" "$*" >> "$here/../scheduler.log"
state="$here/../scheduler.loaded"
case "$1 $2" in
  print*|"--user is-active")
    loaded=
    [ -f "$state" ] && read -r loaded < "$state"
    [ "$loaded" = yes ] ;;
  bootstrap*|"--user enable") printf 'yes\n' > "$state" ;;
  bootout*|"--user disable") printf 'no\n' > "$state" ;;
esac
