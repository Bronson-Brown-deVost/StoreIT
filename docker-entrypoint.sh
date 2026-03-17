#!/bin/sh
set -e

# If the user passed a subcommand (import, version, etc.), run it directly
case "${1:-}" in
    import|version|auto-upgrade|serve)
        exec storeit-server "$@"
        ;;
esac

# Default: auto-upgrade if needed, then serve
storeit-server auto-upgrade
exec storeit-server serve "$@"
