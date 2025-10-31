#!/bin/bash
set -e # Exit with nonzero exit code if anything fails

# Clear cached constants build to force rebuild with current config
echo "Clearing cached constants..."
rm -rf /usr/src/app/packages/constants/dist

# Copy the entire app directory to allow for live updates
echo "Starting Turbo..."

# Use docker env files
cp /usr/src/app/packages/server/.env.docker /usr/src/app/packages/server/.env

cp /usr/src/app/packages/ui/.env.docker /usr/src/app/packages/ui/.env

pnpm run $PNPM_SCRIPT
