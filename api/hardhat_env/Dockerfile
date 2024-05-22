FROM node:18

# Install dependencies
RUN apt-get update && apt-get install -y wget

# Download and install zksolc
RUN wget https://github.com/matter-labs/zksolc-bin/raw/main/linux-amd64/zksolc-linux-amd64-musl-v1.4.1 \
    && chmod +x zksolc-linux-amd64-musl-v1.4.1 \
    && mv zksolc-linux-amd64-musl-v1.4.1 /usr/local/bin/zksolc

WORKDIR /app

COPY package*.json .
COPY hardhat.config.ts .
COPY tsconfig.json .

RUN npm install
