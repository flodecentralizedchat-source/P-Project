const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Vesting", function () {
    let vesting;
    let token;
    let owner;
    let beneficiary;
    let addr2;
    let addrs;

    beforeEach(async function () {
        [owner, beneficiary, addr2, ...addrs] = await ethers.getSigners();
        
        // Deploy PProjectToken first
        const PProjectToken = await ethers.getContractFactory("PProjectToken");
        const totalSupply = ethers.utils.parseEther("350000000"); // 350 million tokens
        const burnRate = ethers.utils.parseEther("0.01"); // 1% burn rate
        const rewardRate = ethers.utils.parseEther("0.005"); // 0.5% reward rate
        
        token = await PProjectToken.deploy(totalSupply, burnRate, rewardRate);
        await token.deployed();
        
        // Deploy Vesting contract
        const Vesting = await ethers.getContractFactory("Vesting");
        vesting = await Vesting.deploy(token.address);
        await vesting.deployed();
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await vesting.owner()).to.equal(owner.address);
        });
        
        it("Should set the correct token address", async function () {
            expect(await vesting.token()).to.equal(token.address);
        });
    });

    describe("Vesting Schedule", function () {
        const startTime = Math.floor(Date.now() / 1000);
        const cliffDuration = 3600; // 1 hour
        const duration = 7200; // 2 hours
        const totalAllocation = ethers.utils.parseEther("10000"); // 10,000 tokens

        beforeEach(async function () {
            // Transfer tokens to vesting contract
            await token.transfer(vesting.address, totalAllocation);
            
            // Create vesting schedule
            await vesting.createVestingSchedule(
                beneficiary.address,
                startTime,
                cliffDuration,
                duration,
                totalAllocation
            );
        });

        it("Should create a vesting schedule", async function () {
            const schedule = await vesting.vestingSchedules(beneficiary.address);
            expect(schedule.beneficiary).to.equal(beneficiary.address);
            expect(schedule.start).to.equal(startTime);
            expect(schedule.cliff).to.equal(startTime + cliffDuration);
            expect(schedule.duration).to.equal(duration);
            expect(schedule.totalAllocation).to.equal(totalAllocation);
            expect(schedule.initialized).to.equal(true);
        });

        it("Should not allow duplicate vesting schedules", async function () {
            await expect(
                vesting.createVestingSchedule(
                    beneficiary.address,
                    startTime,
                    cliffDuration,
                    duration,
                    totalAllocation
                )
            ).to.be.revertedWith("Vesting schedule already exists");
        });

        it("Should calculate vested amount before cliff", async function () {
            const vested = await vesting.vestedAmount(beneficiary.address, startTime + cliffDuration - 1);
            expect(vested).to.equal(0);
        });

        it("Should calculate vested amount after cliff", async function () {
            const timeAfterCliff = startTime + cliffDuration + 1800; // 30 minutes after cliff
            const vested = await vesting.vestedAmount(beneficiary.address, timeAfterCliff);
            // Should be half of total allocation (30 minutes out of 1 hour vesting period)
            expect(vested).to.be.closeTo(totalAllocation.div(2), ethers.utils.parseEther("10")); // Allow small variance
        });

        it("Should calculate vested amount after full duration", async function () {
            const timeAfterFull = startTime + duration + 1000;
            const vested = await vesting.vestedAmount(beneficiary.address, timeAfterFull);
            expect(vested).to.equal(totalAllocation);
        });

        it("Should calculate releasable amount", async function () {
            // Move time forward to after cliff
            await ethers.provider.send("evm_increaseTime", [cliffDuration + 1000]);
            await ethers.provider.send("evm_mine");
            
            const releasable = await vesting.releasable(beneficiary.address);
            expect(releasable).to.be.above(0);
        });

        it("Should release vested tokens", async function () {
            // Move time forward to after cliff
            await ethers.provider.send("evm_increaseTime", [cliffDuration + 1800]); // Halfway through vesting
            await ethers.provider.send("evm_mine");
            
            const initialBalance = await token.balanceOf(beneficiary.address);
            await vesting.connect(beneficiary).release(beneficiary.address);
            const finalBalance = await token.balanceOf(beneficiary.address);
            
            expect(finalBalance).to.be.above(initialBalance);
        });

        it("Should not release tokens for non-beneficiary", async function () {
            await expect(
                vesting.connect(addr2).release(beneficiary.address)
            ).to.be.revertedWith("Not beneficiary");
        });
    });
});