// initialize-contracts.js - Script to initialize contracts after deployment
const { ethers } = require("hardhat");

async function main() {
    // Load deployment information
    const fs = require("fs");
    if (!fs.existsSync("deployment-info.json")) {
        console.log("Deployment info not found. Please run deployment first.");
        return;
    }
    
    const deploymentInfo = JSON.parse(fs.readFileSync("deployment-info.json", "utf8"));
    console.log("Initializing contracts on network:", deploymentInfo.network);
    
    // Get signers
    const [deployer] = await ethers.getSigners();
    console.log("Initializing with account:", deployer.address);
    
    // Get contract instances
    const token = await ethers.getContractAt("PProjectToken", deploymentInfo.contracts.PProjectToken);
    const vesting = await ethers.getContractAt("Vesting", deploymentInfo.contracts.Vesting);
    const treasury = await ethers.getContractAt("Treasury", deploymentInfo.contracts.Treasury);
    const liquidityPool = await ethers.getContractAt("LiquidityPool", deploymentInfo.contracts.LiquidityPool);
    
    // Initialize vesting schedules (example)
    console.log("\nSetting up vesting schedules...");
    const teamAllocation = ethers.utils.parseEther("70000000"); // 20% for team
    const advisorAllocation = ethers.utils.parseEther("17500000"); // 5% for advisors
    
    // Create team vesting schedule (48 months with 12-month cliff)
    const now = Math.floor(Date.now() / 1000);
    await vesting.createVestingSchedule(
        "0xTeamWalletAddress", // Replace with actual team wallet address
        now,
        365 * 24 * 60 * 60, // 12 months cliff
        4 * 365 * 24 * 60 * 60, // 48 months total
        teamAllocation
    );
    
    // Create advisor vesting schedule (24 months with 6-month cliff)
    await vesting.createVestingSchedule(
        "0xAdvisorWalletAddress", // Replace with actual advisor wallet address
        now,
        180 * 24 * 60 * 60, // 6 months cliff
        2 * 365 * 24 * 60 * 60, // 24 months total
        advisorAllocation
    );
    
    console.log("Vesting schedules created.");
    
    // Initialize treasury allocations (example)
    console.log("\nSetting up treasury allocations...");
    await treasury.allocateFunds(
        "Ecosystem Development",
        ethers.utils.parseEther("10000000"), // 10M tokens
        "Funding ecosystem development initiatives"
    );
    
    await treasury.allocateFunds(
        "Community Incentives",
        ethers.utils.parseEther("5000000"), // 5M tokens
        "Community reward programs and incentives"
    );
    
    console.log("Treasury allocations created.");
    
    // Enable auto buybacks
    console.log("\nEnabling auto buybacks...");
    await treasury.setAutoBuybackEnabled(true);
    
    // Add a scheduled buyback
    const futureTimestamp = now + 30 * 24 * 60 * 60; // 30 days from now
    await treasury.addScheduledBuyback(
        futureTimestamp,
        ethers.utils.parseEther("1000000"), // 1M USDT
        ethers.utils.parseEther("0.01") // Target price of $0.01
    );
    
    console.log("Auto buybacks enabled with scheduled buyback.");
    
    // Lock liquidity
    console.log("\nLocking liquidity...");
    await liquidityPool.lockLiquidity(365); // Lock for 1 year
    
    console.log("Liquidity locked for 1 year.");
    
    // Transfer some tokens to liquidity pool for initial liquidity
    console.log("\nSetting up initial liquidity...");
    const initialLiquidity = ethers.utils.parseEther("5000000"); // 5M tokens
    await token.transfer(liquidityPool.address, initialLiquidity);
    
    console.log("Initial liquidity tokens transferred to pool.");
    
    console.log("\nAll contracts initialized successfully!");
    console.log("Next steps:");
    console.log("1. Add USDT to the liquidity pool");
    console.log("2. Add liquidity to the pool");
    console.log("3. List on DEX");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });