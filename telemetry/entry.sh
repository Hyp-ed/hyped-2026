#!/bin/bash
set -e

echo "Clearing cached constants..."
rm -rf /usr/src/app/packages/constants/dist

echo "Starting Turbo..."

cp /usr/src/app/packages/server/.env.docker /usr/src/app/packages/server/.env
cp /usr/src/app/packages/ui/.env.docker /usr/src/app/packages/ui/.env

pnpm run $PNPM_SCRIPT