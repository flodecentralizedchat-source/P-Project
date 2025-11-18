// track-deployment.js - Script to track deployment progress and status
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("P-Project Deployment Tracker");
    console.log("============================\n");
    
    // Check for deployment info file
    const deploymentInfoFile = path.join(process.cwd(), "deployment-info.json");
    
    if (!fs.existsSync(deploymentInfoFile)) {
        console.log("❌ No deployment found. Run deployment first.");
        console.log("\nTo deploy:");
        console.log("1. npm run deploy:testnet  (for testnet)");
        console.log("2. npm run deploy:mainnet  (for mainnet)");
        return;
    }
    
    try {
        const deploymentInfo = JSON.parse(fs.readFileSync(deploymentInfoFile, "utf8"));
        
        console.log("Deployment Information:");
        console.log("=====================");
        console.log("Network:", deploymentInfo.network);
        console.log("Deployed:", new Date(deploymentInfo.deploymentTime).toLocaleString());
        console.log("Deployer:", deploymentInfo.deployer);
        console.log("");
        
        console.log("Deployed Contracts:");
        console.log("==================");
        for (const [contractName, address] of Object.entries(deploymentInfo.contracts)) {
            console.log(`${contractName}: ${address}`);
        }
        
        console.log("\nDeployment Status:");
        console.log("==================");
        console.log("✅ Contracts deployed successfully");
        
        // Check if contracts are verified
        const verifiedFile = path.join(process.cwd(), "verified-contracts.json");
        if (fs.existsSync(verifiedFile)) {
            console.log("✅ Contracts verified");
        } else {
            console.log("⚠️  Contracts not yet verified");
            console.log("   Run: npx hardhat verify --network " + deploymentInfo.network + " <contract-address>");
        }
        
        // Check if contracts are initialized
        const initializedFile = path.join(process.cwd(), "initialized-contracts.json");
        if (fs.existsSync(initializedFile)) {
            console.log("✅ Contracts initialized");
        } else {
            console.log("⚠️  Contracts not yet initialized");
            console.log("   Run: npm run initialize -- --network " + deploymentInfo.network);
        }
        
        // Check if DEX preparation is done
        const dexPreparedFile = path.join(process.cwd(), "dex-prepared.json");
        if (fs.existsSync(dexPreparedFile)) {
            console.log("✅ DEX preparation completed");
        } else {
            console.log("⚠️  DEX preparation not yet completed");
            console.log("   Run: npm run prepare:listing -- --network " + deploymentInfo.network);
        }
        
        console.log("\nNext Steps:");
        console.log("===========");
        
        if (!fs.existsSync(verifiedFile)) {
            console.log("1. Verify contracts on block explorer");
        }
        
        if (!fs.existsSync(initializedFile)) {
            console.log("2. Initialize contracts");
        }
        
        if (!fs.existsSync(dexPreparedFile)) {
            console.log("3. Prepare for DEX listing");
        }
        
        if (fs.existsSync(verifiedFile) && fs.existsSync(initializedFile) && fs.existsSync(dexPreparedFile)) {
            console.log("✅ All deployment steps completed!");
            console.log("🎉 Your contracts are ready for DEX listing!");
        }
        
    } catch (error) {
        console.error("❌ Error reading deployment info:", error.message);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });