FROM node:18

# Install dependencies
RUN apt-get update && apt-get install -y wget

WORKDIR /app

COPY package*.json .
COPY tsconfig.json .

RUN npm install -g pnpm

RUN pnpm install
