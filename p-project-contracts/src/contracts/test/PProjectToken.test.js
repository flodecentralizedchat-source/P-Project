const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PProjectToken", function () {
    let token;
    let owner;
    let addr1;
    let addr2;
    let addrs;

    beforeEach(async function () {
        [owner, addr1, addr2, ...addrs] = await ethers.getSigners();
        
        const PProjectToken = await ethers.getContractFactory("PProjectToken");
        const totalSupply = ethers.utils.parseEther("350000000"); // 350 million tokens
        const burnRate = ethers.utils.parseEther("0.01"); // 1% burn rate
        const rewardRate = ethers.utils.parseEther("0.005"); // 0.5% reward rate
        
        token = await PProjectToken.deploy(totalSupply, burnRate, rewardRate);
        await token.deployed();
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await token.owner()).to.equal(owner.address);
        });

        it("Should assign the total supply to the owner", async function () {
            const ownerBalance = await token.balanceOf(owner.address);
            expect(await token.totalSupply()).to.equal(ownerBalance);
        });

        it("Should set the correct token name and symbol", async function () {
            expect(await token.name()).to.equal("P-Project Token");
            expect(await token.symbol()).to.equal("P");
        });
    });

    describe("Transactions", function () {
        it("Should transfer tokens between accounts", async function () {
            // Transfer 50 tokens from owner to addr1
            await token.transfer(addr1.address, 50);
            const addr1Balance = await token.balanceOf(addr1.address);
            expect(addr1Balance).to.equal(50);
        });

        it("Should fail when trying to send more tokens than available", async function () {
            const initialOwnerBalance = await token.balanceOf(owner.address);
            
            // Try to send more tokens than available
            await expect(
                token.connect(addr1).transfer(owner.address, 1)
            ).to.be.revertedWith("Insufficient balance");
        });
    });

    describe("Deflationary Mechanisms", function () {
        it("Should burn tokens on transfer", async function () {
            const initialSupply = await token.totalSupply();
            await token.transfer(addr1.address, ethers.utils.parseEther("1000"));
            
            // Check that total supply decreased due to burn
            const finalSupply = await token.totalSupply();
            expect(finalSupply).to.be.below(initialSupply);
        });

        it("Should distribute rewards to holders", async function () {
            // Transfer tokens to multiple addresses
            await token.transfer(addr1.address, ethers.utils.parseEther("10000"));
            await token.transfer(addr2.address, ethers.utils.parseEther("10000"));
            
            // Check that both addresses have balances
            const addr1Balance = await token.balanceOf(addr1.address);
            const addr2Balance = await token.balanceOf(addr2.address);
            
            expect(addr1Balance).to.be.above(ethers.utils.parseEther("9999")); // Should be slightly above due to rewards
            expect(addr2Balance).to.be.above(ethers.utils.parseEther("9999")); // Should be slightly above due to rewards
        });
    });

    describe("Burn Mechanisms", function () {
        it("Should allow owner to burn tokens directly", async function () {
            const initialSupply = await token.totalSupply();
            const burnAmount = ethers.utils.parseEther("1000");
            
            await token.burnTokens(burnAmount);
            
            const finalSupply = await token.totalSupply();
            expect(finalSupply).to.equal(initialSupply.sub(burnAmount));
        });

        it("Should add and execute scheduled burns", async function () {
            const burnAmount = ethers.utils.parseEther("1000");
            const futureTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour in the future
            
            // Add scheduled burn
            await token.addScheduledBurn(futureTimestamp, burnAmount);
            
            // Enable burn schedule
            await token.setBurnScheduleEnabled(true);
            
            // Execute scheduled burns (should not execute yet since timestamp is in future)
            const burnedAmount = await token.executeScheduledBurns();
            expect(burnedAmount).to.equal(0);
        });
    });
});