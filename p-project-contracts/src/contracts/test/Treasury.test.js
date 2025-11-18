const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Treasury", function () {
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
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await treasury.owner()).to.equal(owner.address);
        });
        
        it("Should set the correct token address", async function () {
            expect(await treasury.token()).to.equal(token.address);
        });
    });

    describe("Funds Management", function () {
        it("Should add funds to treasury", async function () {
            const initialBalance = await treasury.getBalance("USDT");
            const amount = ethers.utils.parseEther("1000");
            
            await treasury.addFunds("USDT", amount);
            
            const finalBalance = await treasury.getBalance("USDT");
            expect(finalBalance).to.equal(initialBalance.add(amount));
        });

        it("Should allocate funds", async function () {
            // Add funds first
            const amount = ethers.utils.parseEther("1000");
            await treasury.addFunds("USDT", amount);
            
            // Allocate funds
            const allocationAmount = ethers.utils.parseEther("500");
            await treasury.allocateFunds("Marketing", allocationAmount, "Marketing expenses");
            
            const allocations = await treasury.getAllocations();
            expect(allocations.length).to.equal(1);
            expect(allocations[0].name).to.equal("Marketing");
            expect(allocations[0].amount).to.equal(allocationAmount);
            expect(allocations[0].purpose).to.equal("Marketing expenses");
        });

        it("Should not allocate more funds than available", async function () {
            // Add funds first
            const amount = ethers.utils.parseEther("1000");
            await treasury.addFunds("USDT", amount);
            
            // Try to allocate more than available
            const allocationAmount = ethers.utils.parseEther("1500");
            await expect(
                treasury.allocateFunds("Marketing", allocationAmount, "Marketing expenses")
            ).to.be.revertedWith("Insufficient funds");
        });
    });

    describe("Buyback Program", function () {
        const tokenPrice = ethers.utils.parseEther("0.1"); // 0.1 USDT per token
        
        beforeEach(async function () {
            // Add funds to treasury
            const amount = ethers.utils.parseEther("10000"); // 10,000 USDT
            await treasury.addFunds("USDT", amount);
        });

        it("Should execute buyback", async function () {
            const amountToSpend = ethers.utils.parseEther("1000"); // 1,000 USDT
            const initialSupply = await token.totalSupply();
            
            await treasury.executeBuyback(amountToSpend, tokenPrice);
            
            const finalSupply = await token.totalSupply();
            expect(finalSupply).to.be.below(initialSupply);
            
            const buybacks = await treasury.getBuybackRecords();
            expect(buybacks.length).to.equal(1);
            expect(buybacks[0].amountSpent).to.equal(amountToSpend);
        });

        it("Should not execute buyback with insufficient funds", async function () {
            const amountToSpend = ethers.utils.parseEther("20000"); // 20,000 USDT (more than available)
            
            await expect(
                treasury.executeBuyback(amountToSpend, tokenPrice)
            ).to.be.revertedWith("Insufficient funds");
        });

        it("Should add and execute scheduled buyback", async function () {
            const futureTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour in the future
            const amount = ethers.utils.parseEther("1000");
            const targetPrice = ethers.utils.parseEther("0.05"); // 0.05 USDT per token
            
            // Add scheduled buyback
            await treasury.addScheduledBuyback(futureTimestamp, amount, targetPrice);
            
            // Enable auto buyback
            await treasury.setAutoBuybackEnabled(true);
            
            // Execute scheduled buybacks (should not execute yet since timestamp is in future)
            const tokensBought = await treasury.executeScheduledBuybacks(targetPrice);
            expect(tokensBought).to.equal(0);
        });
    });

    describe("Buyback Triggers", function () {
        const currentPrice = ethers.utils.parseEther("0.1"); // 0.1 USDT per token
        
        beforeEach(async function () {
            // Add funds to treasury
            const amount = ethers.utils.parseEther("10000"); // 10,000 USDT
            await treasury.addFunds("USDT", amount);
            
            // Enable auto buyback
            await treasury.setAutoBuybackEnabled(true);
        });

        it("Should add and check buyback triggers", async function () {
            // Add a price drop trigger
            const triggerAmount = ethers.utils.parseEther("1000");
            const threshold = ethers.utils.parseEther("0.05"); // Trigger if price drops to 0.05
            
            await treasury.addBuybackTrigger("Price Drop", "price_drop", threshold, triggerAmount);
            
            // Check triggers
            const triggers = await treasury.getBuybackTriggers();
            expect(triggers.length).to.equal(1);
            expect(triggers[0].triggerName).to.equal("Price Drop");
            expect(triggers[0].condition).to.equal("price_drop");
            expect(triggers[0].threshold).to.equal(threshold);
            expect(triggers[0].amount).to.equal(triggerAmount);
        });

        it("Should execute buyback trigger when condition is met", async function () {
            // Add a price drop trigger
            const triggerAmount = ethers.utils.parseEther("1000");
            const threshold = ethers.utils.parseEther("0.15"); // Trigger if price drops below 0.15
            
            await treasury.addBuybackTrigger("Price Drop", "price_drop", threshold, triggerAmount);
            
            // Check triggers with a price that meets the condition (0.1 < 0.15)
            const tokensBought = await treasury.checkBuybackTriggers(ethers.utils.parseEther("0.1"), "price_drop", 0);
            
            // Since the condition is met, some tokens should be bought
            expect(tokensBought).to.be.above(0);
            
            // Check that the trigger was executed
            const triggers = await treasury.getBuybackTriggers();
            expect(triggers[0].executed).to.equal(true);
        });
    });
});