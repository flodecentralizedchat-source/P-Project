const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Scheduled Operations", function () {
    let treasury;
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
        
        // Deploy Treasury contract
        const signers = [owner.address, addr1.address, addr2.address];
        const required = 2;
        const Treasury = await ethers.getContractFactory("Treasury");
        treasury = await Treasury.deploy(token.address, signers, required);
        await treasury.deployed();
        
        // Transfer token ownership to treasury for burn functionality
        await token.transferOwnership(treasury.address);
    });

    describe("Scheduled Buybacks", function () {
        const tokenPrice = ethers.utils.parseEther("0.1"); // 0.1 USDT per token
        
        beforeEach(async function () {
            // Add funds to treasury
            const amount = ethers.utils.parseEther("10000"); // 10,000 USDT
            await treasury.addFunds("USDT", amount);
        });

        it("Should add and execute scheduled buyback at future timestamp", async function () {
            const futureTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour in the future
            const amount = ethers.utils.parseEther("1000");
            const targetPrice = ethers.utils.parseEther("0.05"); // 0.05 USDT per token
            
            // Add scheduled buyback
            await treasury.addScheduledBuyback(futureTimestamp, amount, targetPrice);
            
            // Enable auto buyback
            await treasury.setAutoBuybackEnabled(true);
            
            // Execute scheduled buybacks (should not execute yet since timestamp is in future)
            const tokensBought = await treasury.executeScheduledBuybacks(tokenPrice);
            expect(tokensBought).to.equal(0);
        });

        it("Should execute scheduled buyback when timestamp is due", async function () {
            // Set a past timestamp to simulate a due buyback
            const pastTimestamp = Math.floor(Date.now() / 1000) - 3600; // 1 hour ago
            const amount = ethers.utils.parseEther("1000");
            const targetPrice = ethers.utils.parseEther("0.05"); // 0.05 USDT per token
            
            // Add scheduled buyback with past timestamp
            await treasury.addScheduledBuyback(pastTimestamp, amount, targetPrice);
            
            // Enable auto buyback
            await treasury.setAutoBuybackEnabled(true);
            
            // Execute scheduled buybacks (should execute since timestamp is in the past)
            const tokensBought = await treasury.executeScheduledBuybacks(tokenPrice);
            expect(tokensBought).to.be.above(0);
            
            // Check that the scheduled buyback was marked as executed
            const scheduledBuybacks = await treasury.getScheduledBuybacks();
            expect(scheduledBuybacks[0].executed).to.equal(true);
        });

        it("Should not execute scheduled buyback when auto buyback is disabled", async function () {
            const pastTimestamp = Math.floor(Date.now() / 1000) - 3600; // 1 hour ago
            const amount = ethers.utils.parseEther("1000");
            const targetPrice = ethers.utils.parseEther("0.05"); // 0.05 USDT per token
            
            // Add scheduled buyback with past timestamp
            await treasury.addScheduledBuyback(pastTimestamp, amount, targetPrice);
            
            // Keep auto buyback disabled (default)
            expect(await treasury.autoBuybackEnabled()).to.equal(false);
            
            // Execute scheduled buybacks (should not execute since auto buyback is disabled)
            const tokensBought = await treasury.executeScheduledBuybacks(tokenPrice);
            expect(tokensBought).to.equal(0);
        });
    });

    describe("Scheduled Burns", function () {
        beforeEach(async function () {
            // Enable trading for burn functionality
            await token.setTradingEnabled(true);
        });

        it("Should add and execute scheduled burn at future timestamp", async function () {
            const futureTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour in the future
            const burnAmount = ethers.utils.parseEther("1000");
            
            // Add scheduled burn
            await token.addScheduledBurn(futureTimestamp, burnAmount);
            
            // Enable burn schedule
            await token.setBurnScheduleEnabled(true);
            
            // Execute scheduled burns (should not execute yet since timestamp is in future)
            const burnedAmount = await token.callStatic.executeScheduledBurns();
            expect(burnedAmount.toString()).to.equal("0");
        });

        it("Should execute scheduled burn when timestamp is due", async function () {
            // Set a past timestamp to simulate a due burn
            const pastTimestamp = Math.floor(Date.now() / 1000) - 3600; // 1 hour ago
            const burnAmount = ethers.utils.parseEther("1000");
            const initialSupply = await token.totalSupply();
            
            // Add scheduled burn with past timestamp
            await token.addScheduledBurn(pastTimestamp, burnAmount);
            
            // Enable burn schedule
            await token.setBurnScheduleEnabled(true);
            
            // Execute scheduled burns (should execute since timestamp is in the past)
            const tx = await token.executeScheduledBurns();
            await tx.wait();
            
            const finalSupply = await token.totalSupply();
            expect(initialSupply.sub(finalSupply)).to.equal(burnAmount);
            
            // Check that the scheduled burn was marked as executed
            // Note: We can't directly access the scheduledBurns array from tests, but we can verify
            // that the burn was executed by checking the supply reduction
        });

        it("Should not execute scheduled burn when burn schedule is disabled", async function () {
            const pastTimestamp = Math.floor(Date.now() / 1000) - 3600; // 1 hour ago
            const burnAmount = ethers.utils.parseEther("1000");
            
            // Add scheduled burn with past timestamp
            await token.addScheduledBurn(pastTimestamp, burnAmount);
            
            // Keep burn schedule disabled (default)
            expect(await token.burnScheduleEnabled()).to.equal(false);
            
            // Execute scheduled burns (should not execute since burn schedule is disabled)
            const burnedAmount = await token.callStatic.executeScheduledBurns();
            expect(burnedAmount.toString()).to.equal("0");
        });
    });
});