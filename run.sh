git clone git@github.com:NethermindEth/zksync-remix-plugin.git

cd zksync-remix-plugin/plugin

pnpm install

screen -S zksync-remix-frontend -d -m pnpm run deploy

cd ../api

cargo build;

screen -S rust-backend -d -m cargo run