// deployment-checklist.js - Script to verify deployment completion
const { ethers } = require("hardhat");

async function main() {
    console.log("P-Project Deployment Verification Checklist");
    console.log("==========================================\n");
    
    // Check if deployment info exists
    const fs = require("fs");
    if (!fs.existsSync("deployment-info.json")) {
        console.log("❌ Deployment info not found. Please run deployment first.");
        return;
    }
    
    const deploymentInfo = JSON.parse(fs.readFileSync("deployment-info.json", "utf8"));
    console.log("✅ Deployment info found");
    console.log("Network:", deploymentInfo.network);
    console.log("Deployed:", deploymentInfo.deploymentTime);
    console.log("");
    
    try {
        // Verify P-Project Token
        console.log("1. Verifying PProjectToken...");
        const token = await ethers.getContractAt("PProjectToken", deploymentInfo.contracts.PProjectToken);
        console.log("   Name:", await token.name());
        console.log("   Symbol:", await token.symbol());
        console.log("   Decimals:", (await token.decimals()).toString());
        console.log("   Total Supply:", ethers.utils.formatEther(await token.totalSupply()));
        console.log("   Owner:", await token.owner());
        console.log("   ✅ PProjectToken verified\n");
        
        // Verify Vesting Contract
        console.log("2. Verifying Vesting...");
        const vesting = await ethers.getContractAt("Vesting", deploymentInfo.contracts.Vesting);
        console.log("   Token Address:", await vesting.token());
        console.log("   Owner:", await vesting.owner());
        console.log("   ✅ Vesting contract verified\n");
        
        // Verify Treasury Contract
        console.log("3. Verifying Treasury...");
        const treasury = await ethers.getContractAt("Treasury", deploymentInfo.contracts.Treasury);
        console.log("   Token Address:", await treasury.token());
        console.log("   Owner:", await treasury.owner());
        console.log("   Multi-sig Required:", (await treasury.multisigRequired()).toString());
        console.log("   ✅ Treasury contract verified\n");
        
        // Verify Liquidity Pool
        console.log("4. Verifying LiquidityPool...");
        const liquidityPool = await ethers.getContractAt("LiquidityPool", deploymentInfo.contracts.LiquidityPool);
        console.log("   Pool ID:", await liquidityPool.poolId());
        console.log("   Fee Tier:", ethers.utils.formatEther(await liquidityPool.feeTier()));
        console.log("   Owner:", await liquidityPool.owner());
        console.log("   ✅ LiquidityPool contract verified\n");
        
        console.log("🎉 All contracts deployed and verified successfully!");
        console.log("\nNext steps:");
        console.log("1. Run 'npx hardhat run scripts/initialize-contracts.js --network", deploymentInfo.network + "'");
        console.log("2. Prepare for DEX listing with 'npx hardhat run scripts/prepare-dex-listing.js --network", deploymentInfo.network + "'");
        console.log("3. Verify contracts on block explorer");
        console.log("4. Set up initial liquidity");
        console.log("5. Submit DEX listing request");
        
    } catch (error) {
        console.error("❌ Error verifying contracts:", error.message);
        console.log("\nTroubleshooting tips:");
        console.log("- Check that deployment-info.json contains correct contract addresses");
        console.log("- Verify network connectivity");
        console.log("- Ensure contracts are deployed to the specified network");
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });