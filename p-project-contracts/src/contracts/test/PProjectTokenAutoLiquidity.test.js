const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PProjectToken Auto-Liquidity", function () {
    let token;
    let owner;
    let addr1;
    let addr2;
    let addrs;
    let mockRouter;
    let mockPair;

    beforeEach(async function () {
        [owner, addr1, addr2, ...addrs] = await ethers.getSigners();
        
        // Deploy mock Uniswap V2 Router
        const MockUniswapV2Router = await ethers.getContractFactory("MockUniswapV2Router");
        mockRouter = await MockUniswapV2Router.deploy();
        await mockRouter.deployed();
        
        // Deploy mock Uniswap V2 Pair
        const MockUniswapV2Pair = await ethers.getContractFactory("MockUniswapV2Pair");
        mockPair = await MockUniswapV2Pair.deploy();
        await mockPair.deployed();
        
        const PProjectToken = await ethers.getContractFactory("PProjectToken");
        const totalSupply = ethers.utils.parseEther("350000000"); // 350 million tokens
        const burnRate = ethers.utils.parseEther("0.01"); // 1% burn rate
        const rewardRate = ethers.utils.parseEther("0.005"); // 0.5% reward rate
        
        token = await PProjectToken.deploy(totalSupply, burnRate, rewardRate);
        await token.deployed();
        
        // Set the Uniswap router
        await token.setUniswapRouter(mockRouter.address);
        
        // Enable trading
        await token.setTradingEnabled(true);
    });

    describe("Auto-Liquidity Settings", function () {
        it("Should have correct default liquidity fees", async function () {
            expect(await token.liquidityFee()).to.equal(ethers.utils.parseEther("0.03")); // 3%
            expect(await token.marketingFee()).to.equal(ethers.utils.parseEther("0.02")); // 2%
        });

        it("Should update liquidity fees", async function () {
            const newLiquidityFee = ethers.utils.parseEther("0.05"); // 5%
            const newMarketingFee = ethers.utils.parseEther("0.03"); // 3%
            
            await token.setLiquidityFees(newLiquidityFee, newMarketingFee);
            
            expect(await token.liquidityFee()).to.equal(newLiquidityFee);
            expect(await token.marketingFee()).to.equal(newMarketingFee);
        });

        it("Should not allow fees to exceed maximum", async function () {
            const maxFee = await token.maxLiquidityFee();
            const excessiveFee = maxFee.add(ethers.utils.parseEther("0.01"));
            
            await expect(
                token.setLiquidityFees(excessiveFee, ethers.utils.parseEther("0.01"))
            ).to.be.revertedWith("Fees exceed maximum");
        });

        it("Should update minimum tokens before swap", async function () {
            const newMinTokens = ethers.utils.parseEther("5000");
            await token.setMinTokensBeforeSwap(newMinTokens);
            expect(await token.minTokensBeforeSwap()).to.equal(newMinTokens);
        });

        it("Should update marketing wallet", async function () {
            await token.setMarketingWallet(addr1.address);
            expect(await token.marketingWallet()).to.equal(addr1.address);
        });

        it("Should enable/disable swap and liquify", async function () {
            await token.setSwapAndLiquifyEnabled(false);
            expect(await token.swapAndLiquifyEnabled()).to.equal(false);
            
            await token.setSwapAndLiquifyEnabled(true);
            expect(await token.swapAndLiquifyEnabled()).to.equal(true);
        });
    });

    describe("Auto-Liquidity Functionality", function () {
        it("Should collect liquidity fees on transfer when trading is enabled", async function () {
            // Transfer tokens to addr1
            const transferAmount = ethers.utils.parseEther("10000");
            await token.transfer(addr1.address, transferAmount);
            
            // Check initial contract balance
            const initialContractBalance = await token.balanceOf(token.address);
            
            // Transfer tokens from addr1 to addr2 (this should trigger fees)
            await token.connect(addr1).transfer(addr2.address, transferAmount);
            
            // Check that fees were collected by the contract
            const finalContractBalance = await token.balanceOf(token.address);
            expect(finalContractBalance).to.be.above(initialContractBalance);
        });

        it("Should not collect fees when trading is disabled", async function () {
            // Disable trading
            await token.setTradingEnabled(false);
            
            // Transfer tokens to addr1
            const transferAmount = ethers.utils.parseEther("10000");
            await token.transfer(addr1.address, transferAmount);
            
            // Check initial contract balance
            const initialContractBalance = await token.balanceOf(token.address);
            
            // Transfer tokens from addr1 to addr2 (this should not trigger fees)
            await token.connect(addr1).transfer(addr2.address, transferAmount);
            
            // Check that no fees were collected by the contract
            const finalContractBalance = await token.balanceOf(token.address);
            expect(finalContractBalance).to.equal(initialContractBalance);
        });

        it("Should send marketing fees to marketing wallet", async function () {
            // Set marketing wallet to addr1
            await token.setMarketingWallet(addr1.address);
            
            // Transfer tokens to addr2
            const transferAmount = ethers.utils.parseEther("10000");
            await token.transfer(addr2.address, transferAmount);
            
            // Check initial marketing wallet balance
            const initialMarketingBalance = await token.balanceOf(addr1.address);
            
            // Transfer tokens from addr2 (this should trigger marketing fees)
            await token.connect(addr2).transfer(owner.address, transferAmount.div(2));
            
            // Check that marketing fees were sent to marketing wallet
            const finalMarketingBalance = await token.balanceOf(addr1.address);
            expect(finalMarketingBalance).to.be.above(initialMarketingBalance);
        });
    });

    describe("Swap and Liquify", function () {
        it("Should execute swap and liquify when threshold is reached", async function () {
            // Set a low threshold for testing
            const lowThreshold = ethers.utils.parseEther("100");
            await token.setMinTokensBeforeSwap(lowThreshold);
            
            // Transfer tokens to generate fees
            const transferAmount = ethers.utils.parseEther("10000");
            await token.transfer(addr1.address, transferAmount);
            
            // This transfer should trigger swap and liquify
            await expect(token.connect(addr1).transfer(addr2.address, transferAmount.div(2)))
                .to.emit(token, "SwapAndLiquify");
        });

        it("Should not execute swap and liquify when disabled", async function () {
            // Disable swap and liquify
            await token.setSwapAndLiquifyEnabled(false);
            
            // Set a low threshold for testing
            const lowThreshold = ethers.utils.parseEther("100");
            await token.setMinTokensBeforeSwap(lowThreshold);
            
            // Transfer tokens to generate fees
            const transferAmount = ethers.utils.parseEther("10000");
            await token.transfer(addr1.address, transferAmount);
            
            // This transfer should not trigger swap and liquify
            await expect(token.connect(addr1).transfer(addr2.address, transferAmount.div(2)))
                .to.not.emit(token, "SwapAndLiquify");
        });
    });
});