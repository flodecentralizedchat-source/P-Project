// verify-deployment.js - Script to verify deployed contracts
const { ethers } = require("hardhat");

async function main() {
    // Load deployment information
    const fs = require("fs");
    if (!fs.existsSync("deployment-info.json")) {
        console.log("Deployment info not found. Please run deployment first.");
        return;
    }
    
    const deploymentInfo = JSON.parse(fs.readFileSync("deployment-info.json", "utf8"));
    console.log("Verifying deployment on network:", deploymentInfo.network);
    
    // Verify P-Project Token
    console.log("\nVerifying PProjectToken...");
    const token = await ethers.getContractAt("PProjectToken", deploymentInfo.contracts.PProjectToken);
    console.log("Token Name:", await token.name());
    console.log("Token Symbol:", await token.symbol());
    console.log("Total Supply:", ethers.utils.formatEther(await token.totalSupply()));
    console.log("Owner:", await token.owner());
    
    // Verify Vesting Contract
    console.log("\nVerifying Vesting...");
    const vesting = await ethers.getContractAt("Vesting", deploymentInfo.contracts.Vesting);
    console.log("Token Address:", await vesting.token());
    console.log("Owner:", await vesting.owner());
    
    // Verify Treasury Contract
    console.log("\nVerifying Treasury...");
    const treasury = await ethers.getContractAt("Treasury", deploymentInfo.contracts.Treasury);
    console.log("Token Address:", await treasury.token());
    console.log("Owner:", await treasury.owner());
    console.log("Multi-sig Required:", (await treasury.multisigRequired()).toString());
    
    // Verify Liquidity Pool
    console.log("\nVerifying LiquidityPool...");
    const liquidityPool = await ethers.getContractAt("LiquidityPool", deploymentInfo.contracts.LiquidityPool);
    console.log("Pool ID:", await liquidityPool.poolId());
    console.log("Fee Tier:", ethers.utils.formatEther(await liquidityPool.feeTier()));
    console.log("Owner:", await liquidityPool.owner());
    
    console.log("\nAll contracts verified successfully!");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });