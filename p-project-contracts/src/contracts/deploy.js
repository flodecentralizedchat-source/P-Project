// deploy.js - Deployment script for P-Project contracts
const { ethers } = require("hardhat");

async function main() {
    // Get accounts
    const [deployer] = await ethers.getSigners();
    
    console.log("Deploying contracts with the account:", deployer.address);
    console.log("Account balance:", (await deployer.getBalance()).toString());
    
    // Deploy P-Project Token
    console.log("\nDeploying PProjectToken...");
    const PProjectToken = await ethers.getContractFactory("PProjectToken");
    const totalSupply = ethers.utils.parseEther("350000000"); // 350 million tokens
    const burnRate = ethers.utils.parseEther("0.01"); // 1% burn rate
    const rewardRate = ethers.utils.parseEther("0.005"); // 0.5% reward rate
    
    const token = await PProjectToken.deploy(totalSupply, burnRate, rewardRate);
    await token.deployed();
    console.log("PProjectToken deployed to:", token.address);
    
    // Deploy Vesting Contract
    console.log("\nDeploying Vesting...");
    const Vesting = await ethers.getContractFactory("Vesting");
    const vesting = await Vesting.deploy(token.address);
    await vesting.deployed();
    console.log("Vesting deployed to:", vesting.address);
    
    // Deploy Treasury Contract
    console.log("\nDeploying Treasury...");
    const Treasury = await ethers.getContractFactory("Treasury");
    const signers = [deployer.address]; // Add more signers as needed
    const requiredSignatures = 1;
    
    const treasury = await Treasury.deploy(token.address, signers, requiredSignatures);
    await treasury.deployed();
    console.log("Treasury deployed to:", treasury.address);
    
    // Deploy Liquidity Pool
    console.log("\nDeploying LiquidityPool...");
    const LiquidityPool = await ethers.getContractFactory("LiquidityPool");
    // For this example, we'll use a mock USDT address
    // In practice, you would use the actual USDT contract address
    const mockUSDT = "0x0000000000000000000000000000000000000000"; // Replace with actual USDT address
    const feeTier = ethers.utils.parseEther("0.003"); // 0.3% fee
    const lockDuration = 365; // 1 year lock
    
    const liquidityPool = await LiquidityPool.deploy(
        token.address,
        mockUSDT,
        "P-USDT-POOL",
        feeTier,
        lockDuration
    );
    await liquidityPool.deployed();
    console.log("LiquidityPool deployed to:", liquidityPool.address);
    
    // Transfer ownership of contracts to treasury (if needed)
    console.log("\nSetting up contract relationships...");
    
    // Transfer some tokens to treasury for buybacks
    const treasuryAllocation = ethers.utils.parseEther("35000000"); // 10% of total supply
    await token.transfer(treasury.address, treasuryAllocation);
    console.log("Transferred", ethers.utils.formatEther(treasuryAllocation), "tokens to Treasury");
    
    // Verify deployment
    console.log("\nDeployment Summary:");
    console.log("PProjectToken:", token.address);
    console.log("Vesting:", vesting.address);
    console.log("Treasury:", treasury.address);
    console.log("LiquidityPool:", liquidityPool.address);
    
    // Save deployment addresses to a file
    const fs = require("fs");
    const deploymentInfo = {
        network: network.name,
        deployer: deployer.address,
        contracts: {
            PProjectToken: token.address,
            Vesting: vesting.address,
            Treasury: treasury.address,
            LiquidityPool: liquidityPool.address
        },
        deploymentTime: new Date().toISOString()
    };
    
    fs.writeFileSync(
        "deployment-info.json",
        JSON.stringify(deploymentInfo, null, 2)
    );
    console.log("\nDeployment information saved to deployment-info.json");
}

// Run the deployment
main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });