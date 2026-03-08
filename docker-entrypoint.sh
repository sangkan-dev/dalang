#!/bin/sh

# Initialize the Dalang environment (installs skills etc)
dalang init

# Execute the command passed as arguments (CMD)
exec "$@"
