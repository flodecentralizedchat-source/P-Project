// prepare-dex-listing.js - Script to prepare contracts for DEX listing
const { ethers } = require("hardhat");

async function main() {
    // Load deployment information
    const fs = require("fs");
    if (!fs.existsSync("deployment-info.json")) {
        console.log("Deployment info not found. Please run deployment first.");
        return;
    }
    
    const deploymentInfo = JSON.parse(fs.readFileSync("deployment-info.json", "utf8"));
    console.log("Preparing DEX listing on network:", deploymentInfo.network);
    
    // Get signers
    const [deployer] = await ethers.getSigners();
    console.log("Preparing with account:", deployer.address);
    
    // Get contract instances
    const token = await ethers.getContractAt("PProjectToken", deploymentInfo.contracts.PProjectToken);
    const liquidityPool = await ethers.getContractAt("LiquidityPool", deploymentInfo.contracts.LiquidityPool);
    
    // Check current balances
    console.log("\nCurrent balances:");
    const tokenBalance = await token.balanceOf(deployer.address);
    console.log("Deployer P token balance:", ethers.utils.formatEther(tokenBalance));
    
    const poolTokenBalance = await token.balanceOf(liquidityPool.address);
    console.log("Pool P token balance:", ethers.utils.formatEther(poolTokenBalance));
    
    // Prepare initial liquidity as per tokenomics
    console.log("\nPreparing initial liquidity...");
    const initialTokenLiquidity = ethers.utils.parseEther("5000000"); // 5M tokens
    const initialUSDTLiquidity = ethers.utils.parseEther("50000"); // $50,000 USDT
    
    console.log("Required P tokens for liquidity:", ethers.utils.formatEther(initialTokenLiquidity));
    console.log("Required USDT for liquidity:", ethers.utils.formatEther(initialUSDTLiquidity));
    
    // Check if we have enough tokens
    if (tokenBalance.lt(initialTokenLiquidity)) {
        console.log("Warning: Insufficient P tokens for initial liquidity.");
        console.log("You need to transfer", ethers.utils.formatEther(initialTokenLiquidity.sub(tokenBalance)), "more tokens.");
    } else {
        console.log("✓ Sufficient P tokens available for initial liquidity.");
    }
    
    // Add liquidity to pool (this would require USDT to be available too)
    console.log("\nTo add liquidity to the pool, you need to:");
    console.log("1. Transfer", ethers.utils.formatEther(initialTokenLiquidity), "P tokens to the pool");
    console.log("2. Transfer", ethers.utils.formatEther(initialUSDTLiquidity), "USDT to the pool");
    console.log("3. Call addLiquidity() on the pool contract");
    
    // Example of how to add liquidity (commented out as it requires USDT)
    /*
    await token.transfer(liquidityPool.address, initialTokenLiquidity);
    
    // This would require USDT contract interaction
    // const usdt = await ethers.getContractAt("IERC20", "USDT_CONTRACT_ADDRESS");
    // await usdt.transfer(liquidityPool.address, initialUSDTLiquidity);
    
    await liquidityPool.addLiquidity(
        deployer.address,
        initialTokenLiquidity,
        initialUSDTLiquidity,
        365 // Lock for 1 year
    );
    */
    
    // Verify contract parameters
    console.log("\nVerifying contract parameters for DEX listing:");
    console.log("Token name:", await token.name());
    console.log("Token symbol:", await token.symbol());
    console.log("Token decimals:", await token.decimals());
    console.log("Total supply:", ethers.utils.formatEther(await token.totalSupply()));
    
    console.log("\nPool parameters:");
    console.log("Pool ID:", await liquidityPool.poolId());
    console.log("Fee tier:", ethers.utils.formatEther(await liquidityPool.feeTier()) + " (", 
                (await liquidityPool.feeTier()).div(ethers.utils.parseEther("0.01")).toString(), "%)");
    console.log("Liquidity locked:", await liquidityPool.liquidityLocked());
    
    console.log("\nDEX listing preparation complete!");
    console.log("\nNext steps for DEX listing:");
    console.log("1. Ensure you have sufficient P tokens and USDT for initial liquidity");
    console.log("2. Transfer tokens to the liquidity pool");
    console.log("3. Add liquidity to the pool");
    console.log("4. Verify contracts on block explorer");
    console.log("5. Submit pool creation request to DEX");
    console.log("6. Announce listing to community");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });