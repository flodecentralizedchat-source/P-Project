const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("LiquidityPool", function () {
    let liquidityPool;
    let token;
    let owner;
    let addr1;
    let addr2;
    let addrs;

    beforeEach(async function () {
        [owner, addr1, addr2, ...addrs] = await ethers.getSigners();
        
        // Deploy PProjectToken first
        const PProjectToken = await ethers.getContractFactory("PProjectToken");
        const totalSupply = ethers.utils.parseEther("350000000"); // 350 million tokens
        const burnRate = ethers.utils.parseEther("0.01"); // 1% burn rate
        const rewardRate = ethers.utils.parseEther("0.005"); // 0.5% reward rate
        
        token = await PProjectToken.deploy(totalSupply, burnRate, rewardRate);
        await token.deployed();
        
        // Deploy LiquidityPool contract
        const feeTier = ethers.utils.parseEther("0.003"); // 0.3% fee
        const lockDuration = 365; // 365 days
        const LiquidityPool = await ethers.getContractFactory("LiquidityPool");
        liquidityPool = await LiquidityPool.deploy(
            token.address,
            addr1.address, // USDT address (using addr1 as placeholder)
            "P-USDT-LP",
            feeTier,
            lockDuration
        );
        await liquidityPool.deployed();
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await liquidityPool.owner()).to.equal(owner.address);
        });
        
        it("Should set the correct token address", async function () {
            expect(await liquidityPool.token()).to.equal(token.address);
        });
        
        it("Should set the correct fee tier", async function () {
            expect(await liquidityPool.feeTier()).to.equal(ethers.utils.parseEther("0.003"));
        });
        
        it("Should set the correct pool ID", async function () {
            expect(await liquidityPool.poolId()).to.equal("P-USDT-LP");
        });
    });

    describe("Liquidity Management", function () {
        const tokenAmount = ethers.utils.parseEther("10000"); // 10,000 P tokens
        const usdtAmount = ethers.utils.parseEther("1000"); // 1,000 USDT
        const durationDays = 365; // 1 year

        beforeEach(async function () {
            // Transfer tokens to liquidity pool
            await token.transfer(liquidityPool.address, tokenAmount);
        });

        it("Should add liquidity", async function () {
            const initialTotalLiquidity = await liquidityPool.totalLiquidity();
            
            await liquidityPool.addLiquidity(
                addr1.address,
                tokenAmount,
                usdtAmount,
                durationDays
            );
            
            const finalTotalLiquidity = await liquidityPool.totalLiquidity();
            expect(finalTotalLiquidity).to.be.above(initialTotalLiquidity);
            
            // Check liquidity position
            const position = await liquidityPool.liquidityPositions(addr1.address);
            expect(position.user).to.equal(addr1.address);
            expect(position.tokenAmount).to.equal(tokenAmount);
            expect(position.usdtAmount).to.equal(usdtAmount);
            expect(position.durationDays).to.equal(durationDays);
        });

        it("Should not add liquidity with zero amounts", async function () {
            await expect(
                liquidityPool.addLiquidity(
                    addr1.address,
                    0,
                    usdtAmount,
                    durationDays
                )
            ).to.be.revertedWith("Amounts must be positive");
            
            await expect(
                liquidityPool.addLiquidity(
                    addr1.address,
                    tokenAmount,
                    0,
                    durationDays
                )
            ).to.be.revertedWith("Amounts must be positive");
        });

        it("Should not add liquidity with zero duration", async function () {
            await expect(
                liquidityPool.addLiquidity(
                    addr1.address,
                    tokenAmount,
                    usdtAmount,
                    0
                )
            ).to.be.revertedWith("Duration must be positive");
        });

        it("Should remove liquidity", async function () {
            // First add liquidity
            await liquidityPool.addLiquidity(
                addr1.address,
                tokenAmount,
                usdtAmount,
                durationDays
            );
            
            const liquidityAmount = await liquidityPool.totalLiquidity();
            const halfLiquidity = liquidityAmount.div(2);
            
            const initialTotalLiquidity = await liquidityPool.totalLiquidity();
            
            // Remove half the liquidity
            await liquidityPool.removeLiquidity(addr1.address, halfLiquidity);
            
            const finalTotalLiquidity = await liquidityPool.totalLiquidity();
            expect(finalTotalLiquidity).to.equal(initialTotalLiquidity.sub(halfLiquidity));
        });

        it("Should not remove more liquidity than available", async function () {
            // First add liquidity
            await liquidityPool.addLiquidity(
                addr1.address,
                tokenAmount,
                usdtAmount,
                durationDays
            );
            
            const excessiveLiquidity = (await liquidityPool.totalLiquidity()).add(1);
            
            await expect(
                liquidityPool.removeLiquidity(addr1.address, excessiveLiquidity)
            ).to.be.revertedWith("Insufficient liquidity");
        });
    });

    describe("Swap Functionality", function () {
        const tokenAmount = ethers.utils.parseEther("10000"); // 10,000 P tokens
        const usdtAmount = ethers.utils.parseEther("1000"); // 1,000 USDT

        beforeEach(async function () {
            // Transfer tokens to liquidity pool
            await token.transfer(liquidityPool.address, tokenAmount);
            
            // Add initial liquidity
            await liquidityPool.addLiquidity(
                owner.address,
                tokenAmount,
                usdtAmount,
                365
            );
        });

        it("Should calculate swap output for P to USDT", async function () {
            const amountIn = ethers.utils.parseEther("100"); // 100 P tokens
            const [amountOut, fee] = await liquidityPool.calculateSwapOutput("P", amountIn);
            
            expect(amountOut).to.be.above(0);
            expect(fee).to.be.above(0);
        });

        it("Should calculate swap output for USDT to P", async function () {
            const amountIn = ethers.utils.parseEther("10"); // 10 USDT
            const [amountOut, fee] = await liquidityPool.calculateSwapOutput("USDT", amountIn);
            
            expect(amountOut).to.be.above(0);
            expect(fee).to.be.above(0);
        });

        it("Should not calculate swap output with zero amount", async function () {
            await expect(
                liquidityPool.calculateSwapOutput("P", 0)
            ).to.be.revertedWith("Amount must be positive");
        });

        it("Should execute swap", async function () {
            const amountIn = ethers.utils.parseEther("100"); // 100 P tokens
            const initialVolume = await liquidityPool.totalVolume();
            
            await liquidityPool.swap(addr1.address, "P", amountIn);
            
            const finalVolume = await liquidityPool.totalVolume();
            expect(finalVolume).to.equal(initialVolume.add(amountIn));
        });
    });

    describe("Liquidity Locking", function () {
        it("Should lock liquidity", async function () {
            const durationDays = 365; // 1 year
            
            await liquidityPool.lockLiquidity(durationDays);
            
            expect(await liquidityPool.liquidityLocked()).to.equal(true);
            expect(await liquidityPool.lockDuration()).to.equal(durationDays);
        });

        it("Should unlock liquidity", async function () {
            // First lock liquidity
            await liquidityPool.lockLiquidity(365);
            
            // Then unlock
            await liquidityPool.unlockLiquidity();
            
            expect(await liquidityPool.liquidityLocked()).to.equal(false);
        });

        it("Should not remove liquidity when locked", async function () {
            const tokenAmount = ethers.utils.parseEther("10000"); // 10,000 P tokens
            const usdtAmount = ethers.utils.parseEther("1000"); // 1,000 USDT
            
            // Transfer tokens to liquidity pool
            await token.transfer(liquidityPool.address, tokenAmount);
            
            // Add liquidity
            await liquidityPool.addLiquidity(
                addr1.address,
                tokenAmount,
                usdtAmount,
                365
            );
            
            // Lock liquidity
            await liquidityPool.lockLiquidity(365);
            
            // Try to remove liquidity (should fail)
            const liquidityAmount = await liquidityPool.totalLiquidity();
            await expect(
                liquidityPool.removeLiquidity(addr1.address, liquidityAmount)
            ).to.be.revertedWith("Liquidity is locked");
        });
    });
});