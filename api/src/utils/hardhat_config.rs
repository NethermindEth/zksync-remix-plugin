use crate::utils::lib::timestamp;
use rand::Rng;
use rocket::serde::json::serde_json;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HardhatConfig {
    #[serde(skip)]
    pub name: String,
    pub zksolc: ZksolcConfig,
    pub solidity: SolidityConfig,
    pub paths: PathsConfig,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct ZksolcConfig {
    pub version: String,
    pub settings: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SolidityConfig {
    pub version: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct PathsConfig {
    pub sources: String,
    pub artifacts: String,
}

#[derive(Default)]
pub struct HardhatConfigBuilder {
    config: HardhatConfig,
}

impl Default for HardhatConfig {
    fn default() -> Self {
        Self {
            name: Self::generate_random_name(),
            zksolc: ZksolcConfig {
                version: "latest".to_string(),
                settings: serde_json::json!({}),
            },
            solidity: SolidityConfig {
                version: "0.8.24".to_string(),
            },
            paths: PathsConfig {
                sources: "./contracts".to_string(),
                artifacts: "./artifacts".to_string(),
            },
        }
    }
}

impl HardhatConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_string_config(&self) -> String {
        let config_prefix_js = r#"
import { HardhatUserConfig } from "hardhat/config";

import "@matterlabs/hardhat-zksync-deploy";
import "@matterlabs/hardhat-zksync-solc";
import "@matterlabs/hardhat-zksync-verify";

export const zkSyncTestnet = process.env.NODE_ENV == "test"
? {
    url: "http://127.0.0.1:8011",
    ethNetwork: "http://127.0.0.1:8045",
    zksync: true,
  }
: {
    url: "https://sepolia.era.zksync.dev",
    ethNetwork: "sepolia",
    zksync: true,
    verifyURL: "https://explorer.sepolia.era.zksync.dev/contract_verification"
  };

export const zkSyncMainnet = {
    url: "https://mainnet.era.zksync.io",
    ethNetwork: "mainnet",
    zksync: true,
    verifyURL: "https://zksync2-mainnet-explorer.zksync.io/contract_verification"
  };
"#;

        let config = format!(
            r#"{}
const config: HardhatUserConfig = {{
  zksolc: {{
    version: "{}",
    settings: {{}},
  }},
  defaultNetwork: "zkSyncTestnet",
  networks: {{
    hardhat: {{
      zksync: false,
    }},
    zkSyncTestnet,
    zkSyncMainnet,
  }},
  solidity: {{
    version: "0.8.24",
  }},
  // path to the directory with contracts
  paths: {{
    sources: "{}",
    artifacts: "{}",
  }},
}};

export default config;
"#,
            config_prefix_js, self.zksolc.version, self.paths.sources, self.paths.artifacts
        );

        config
    }

    pub fn generate_random_name() -> String {
        let mut rng = rand::thread_rng();
        let rand_string: Vec<u8> = std::iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(10)
            .collect();
        format!(
            "hardhat-{}-{}.config.ts",
            timestamp(),
            String::from_utf8(rand_string).unwrap_or("".to_string())
        )
    }
}

impl HardhatConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn zksolc_version(&mut self, version: &str) -> &mut Self {
        self.config.zksolc.version = version.to_string();
        self
    }

    pub fn solidity_version(&mut self, version: &str) -> &mut Self {
        self.config.solidity.version = version.to_string();
        self
    }

    pub fn sources_path(&mut self, path: &str) -> &mut Self {
        self.config.paths.sources = path.to_string();
        self
    }

    pub fn artifacts_path(&mut self, path: &str) -> &mut Self {
        self.config.paths.artifacts = path.to_string();
        self
    }

    pub fn build(&self) -> HardhatConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_random_name() {
        for _ in 1..100 {
            let name = HardhatConfig::generate_random_name();

            println!("Random name: {}", name);
        }
    }
}
