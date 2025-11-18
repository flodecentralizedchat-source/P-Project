// validate-config.js - Script to validate deployment configuration
const fs = require("fs");
const path = require("path");
require("dotenv").config();

async function main() {
    console.log("P-Project Configuration Validator");
    console.log("===============================\n");
    
    // Check environment variables
    console.log("1. Validating environment variables...");
    
    const requiredEnvVars = [
        "PRIVATE_KEY",
        "BSC_TESTNET_URL",
        "BSC_MAINNET_URL"
    ];
    
    let allEnvVarsPresent = true;
    
    for (const envVar of requiredEnvVars) {
        if (process.env[envVar]) {
            console.log(`   ✅ ${envVar} is set`);
        } else {
            console.log(`   ❌ ${envVar} is missing`);
            allEnvVarsPresent = false;
        }
    }
    
    if (!allEnvVarsPresent) {
        console.log("\n❌ Some required environment variables are missing.");
        console.log("   Please check your .env file and ensure all required variables are set.");
        return;
    }
    
    console.log("   ✅ All required environment variables are present\n");
    
    // Validate private key format
    console.log("2. Validating private key format...");
    
    const privateKey = process.env.PRIVATE_KEY;
    if (privateKey.startsWith("0x") && privateKey.length === 66) {
        console.log("   ✅ Private key format is valid");
    } else if (!privateKey.startsWith("0x") && privateKey.length === 64) {
        console.log("   ✅ Private key format is valid (without 0x prefix)");
    } else {
        console.log("   ❌ Private key format is invalid");
        console.log("   Private key should be 64 hexadecimal characters (32 bytes)");
        console.log("   It may optionally include '0x' prefix");
    }
    
    // Validate RPC endpoints
    console.log("\n3. Validating RPC endpoints...");
    
    const rpcEndpoints = [
        { name: "BSC Testnet", url: process.env.BSC_TESTNET_URL },
        { name: "BSC Mainnet", url: process.env.BSC_MAINNET_URL }
    ];
    
    for (const endpoint of rpcEndpoints) {
        if (endpoint.url && endpoint.url.startsWith("http")) {
            console.log(`   ✅ ${endpoint.name} URL format is valid`);
        } else if (endpoint.url) {
            console.log(`   ⚠️  ${endpoint.name} URL format may be invalid`);
            console.log(`      URL: ${endpoint.url}`);
        } else {
            console.log(`   ❌ ${endpoint.name} URL is missing`);
        }
    }
    
    // Check for optional Etherscan API key
    console.log("\n4. Checking optional configurations...");
    
    if (process.env.ETHERSCAN_API_KEY) {
        console.log("   ✅ Etherscan API key is set (optional but recommended)");
    } else {
        console.log("   ⚠️  Etherscan API key is not set");
        console.log("      This is optional but recommended for contract verification");
    }
    
    // Check Hardhat configuration
    console.log("\n5. Validating Hardhat configuration...");
    
    const hardhatConfigFile = path.join(process.cwd(), "hardhat.config.js");
    if (fs.existsSync(hardhatConfigFile)) {
        console.log("   ✅ hardhat.config.js found");
        
        try {
            const hardhatConfig = require(hardhatConfigFile);
            
            // Check if networks are configured
            if (hardhatConfig.networks) {
                console.log("   ✅ Network configurations found");
                
                const requiredNetworks = ["bscTestnet", "bsc"];
                for (const network of requiredNetworks) {
                    if (hardhatConfig.networks[network]) {
                        console.log(`   ✅ ${network} network configuration found`);
                    } else {
                        console.log(`   ⚠️  ${network} network configuration missing`);
                    }
                }
            } else {
                console.log("   ⚠️  No network configurations found");
            }
            
        } catch (error) {
            console.log("   ❌ Error reading hardhat.config.js:", error.message);
        }
    } else {
        console.log("   ❌ hardhat.config.js not found");
    }
    
    // Check package.json
    console.log("\n6. Validating package.json...");
    
    const packageJsonFile = path.join(process.cwd(), "package.json");
    if (fs.existsSync(packageJsonFile)) {
        console.log("   ✅ package.json found");
        
        try {
            const packageJson = JSON.parse(fs.readFileSync(packageJsonFile, "utf8"));
            
            // Check if required scripts are present
            const requiredScripts = [
                "compile",
                "test",
                "deploy:testnet",
                "deploy:mainnet"
            ];
            
            if (packageJson.scripts) {
                console.log("   ✅ Scripts section found");
                
                for (const script of requiredScripts) {
                    if (packageJson.scripts[script]) {
                        console.log(`   ✅ ${script} script found`);
                    } else {
                        console.log(`   ⚠️  ${script} script missing`);
                    }
                }
            } else {
                console.log("   ⚠️  Scripts section missing");
            }
            
        } catch (error) {
            console.log("   ❌ Error reading package.json:", error.message);
        }
    } else {
        console.log("   ❌ package.json not found");
    }
    
    console.log("\n🎉 Configuration validation completed!");
    
    if (allEnvVarsPresent) {
        console.log("\n✅ Your configuration appears to be valid!");
        console.log("You're ready to proceed with deployment.");
    } else {
        console.log("\n⚠️  Some configuration issues were detected.");
        console.log("Please address the issues above before proceeding with deployment.");
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });