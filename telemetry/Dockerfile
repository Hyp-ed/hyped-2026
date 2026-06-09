FROM node:18.17.0

# Create app directory
WORKDIR /usr/src/app

# Install pnpm
RUN npm install -g pnpm

# Install dependencies
COPY telemetry/package.json ./
COPY telemetry/packages/constants/package.json ./packages/constants/
COPY telemetry/packages/server/package.json ./packages/server/
COPY telemetry/packages/types/package.json ./packages/types/
COPY telemetry/packages/ui/package.json ./packages/ui/
COPY telemetry/packages/public-app/package.json ./packages/public-app/
COPY telemetry/packages/tsconfig/package.json ./packages/tsconfig/
COPY config ./config
COPY telemetry/pnpm-lock.yaml ./
COPY telemetry/pnpm-workspace.yaml ./
COPY telemetry/patches ./patches
COPY telemetry/turbo.json ./
RUN pnpm install --frozen-lockfile

COPY telemetry/packages ./packages

# Expose ports
EXPOSE 5173
EXPOSE 3000
EXPOSE 3001
EXPOSE 8086
EXPOSE 9323

# Entrypoint
COPY telemetry/entry.sh ./
RUN chmod +x entry.sh

ENTRYPOINT [ "/usr/src/app/entry.sh" ]