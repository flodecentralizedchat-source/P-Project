require("@nomiclabs/hardhat-waffle");
require("@nomiclabs/hardhat-etherscan");
require("dotenv").config();

// This is a sample Hardhat network config. Replace with your own network details.
module.exports = {
  solidity: {
    compilers: [
      {
        version: "0.8.20",
        settings: {
          optimizer: {
            enabled: true,
            runs: 200
          }
        }
      }
    ]
  },
  paths: {
    sources: "./contracts", // Source files in the contracts directory
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts"
  },
  networks: {
    hardhat: {
      chainId: 1337
    },
    // BSC Testnet
    bscTestnet: {
      url: process.env.BSC_TESTNET_URL || "https://data-seed-prebsc-1-s1.binance.org:8545",
      chainId: 97,
      gasPrice: 20000000000,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // BSC Mainnet
    bsc: {
      url: process.env.BSC_MAINNET_URL || "https://bsc-dataseed.binance.org/",
      chainId: 56,
      gasPrice: 20000000000,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Ethereum Goerli Testnet
    goerli: {
      url: process.env.GOERLI_URL || "https://goerli.infura.io/v3/YOUR_INFURA_PROJECT_ID",
      chainId: 5,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Ethereum Mainnet
    mainnet: {
      url: process.env.MAINNET_URL || "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID",
      chainId: 1,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Ethereum Sepolia (testnet)
    sepolia: {
      url: process.env.SEPOLIA_URL || "https://sepolia.infura.io/v3/YOUR_INFURA_PROJECT_ID",
      chainId: 11155111,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Polygon Mainnet
    polygon: {
      url: process.env.POLYGON_URL || "https://polygon-rpc.com",
      chainId: 137,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Polygon Amoy (testnet)
    amoy: {
      url: process.env.AMOY_URL || "https://rpc-amoy.polygon.technology",
      chainId: 80002,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Arbitrum One
    arbitrum: {
      url: process.env.ARBITRUM_URL || "https://arb1.arbitrum.io/rpc",
      chainId: 42161,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Arbitrum Sepolia (testnet)
    arbitrumSepolia: {
      url: process.env.ARBITRUM_SEPOLIA_URL || "https://sepolia-rollup.arbitrum.io/rpc",
      chainId: 421614,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Optimism Mainnet
    optimism: {
      url: process.env.OPTIMISM_URL || "https://mainnet.optimism.io",
      chainId: 10,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    },
    // Optimism Sepolia (testnet)
    optimismSepolia: {
      url: process.env.OPTIMISM_SEPOLIA_URL || "https://sepolia.optimism.io",
      chainId: 11155420,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []
    }
  },
  etherscan: {
    // Your API key for Etherscan
    // Obtain one at https://bscscan.com/myapikey for BSC or https://etherscan.io/myapikey for Ethereum
    apiKey: process.env.ETHERSCAN_API_KEY || "YOUR_ETHERSCAN_API_KEY"
  }
};