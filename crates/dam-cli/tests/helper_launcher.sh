#!/bin/sh
exec sh "${0%/*/*}/helper-scripts/${0##*/}" "$@"
