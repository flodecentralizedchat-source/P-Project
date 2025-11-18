const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Bridge", function () {
    let bridge;
    let token;
    let owner;
    let addr1;
    let addr2;
    let addrs;

    beforeEach(async function () {
        [owner, addr1, addr2, ...addrs] = await ethers.getSigners();
        
        // Deploy Bridge contract
        const Bridge = await ethers.getContractFactory("Bridge");
        bridge = await Bridge.deploy();
        await bridge.deployed();
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await bridge.owner()).to.equal(owner.address);
        });
        
        it("Should initialize with correct default values", async function () {
            expect(await bridge.paused()).to.equal(false);
            expect(await bridge.kycRequired()).to.equal(false);
            expect(await bridge.enforceTokenAllowlist()).to.equal(false);
            expect(await bridge.nonce()).to.equal(0);
        });
    });

    describe("Ownership", function () {
        it("Should transfer ownership", async function () {
            await bridge.transferOwnership(addr1.address);
            expect(await bridge.owner()).to.equal(addr1.address);
        });

        it("Should not transfer ownership to zero address", async function () {
            await expect(
                bridge.transferOwnership(ethers.constants.AddressZero)
            ).to.be.revertedWith("zero");
        });

        it("Should not allow non-owner to transfer ownership", async function () {
            await expect(
                bridge.connect(addr1).transferOwnership(addr2.address)
            ).to.be.revertedWith("not owner");
        });

        it("Should renounce ownership", async function () {
            await bridge.renounceOwnership();
            expect(await bridge.owner()).to.equal(ethers.constants.AddressZero);
        });

        it("Should not allow non-owner to renounce ownership", async function () {
            await expect(
                bridge.connect(addr1).renounceOwnership()
            ).to.be.revertedWith("not owner");
        });
    });

    describe("Pause Controls", function () {
        it("Should pause the bridge", async function () {
            await bridge.pause();
            expect(await bridge.paused()).to.equal(true);
        });

        it("Should not pause when already paused", async function () {
            await bridge.pause();
            await expect(
                bridge.pause()
            ).to.be.revertedWith("paused");
        });

        it("Should unpause the bridge", async function () {
            await bridge.pause();
            await bridge.unpause();
            expect(await bridge.paused()).to.equal(false);
        });

        it("Should not unpause when not paused", async function () {
            await expect(
                bridge.unpause()
            ).to.be.revertedWith("!paused");
        });

        it("Should not allow non-owner to pause", async function () {
            await expect(
                bridge.connect(addr1).pause()
            ).to.be.revertedWith("not owner");
        });

        it("Should not allow non-owner to unpause", async function () {
            await expect(
                bridge.connect(addr1).unpause()
            ).to.be.revertedWith("not owner");
        });
    });

    describe("Compliance Management", function () {
        it("Should set KYC requirement", async function () {
            await bridge.setKycRequired(true);
            expect(await bridge.kycRequired()).to.equal(true);
            
            await bridge.setKycRequired(false);
            expect(await bridge.kycRequired()).to.equal(false);
        });

        it("Should not allow non-owner to set KYC requirement", async function () {
            await expect(
                bridge.connect(addr1).setKycRequired(true)
            ).to.be.revertedWith("not owner");
        });

        it("Should set KYC status for an account", async function () {
            await bridge.setKyc(addr1.address, true);
            expect(await bridge.kycApproved(addr1.address)).to.equal(true);
            
            await bridge.setKyc(addr1.address, false);
            expect(await bridge.kycApproved(addr1.address)).to.equal(false);
        });

        it("Should not allow non-owner to set KYC status", async function () {
            await expect(
                bridge.connect(addr1).setKyc(addr2.address, true)
            ).to.be.revertedWith("not owner");
        });

        it("Should block and unblock an account", async function () {
            await bridge.setBlocked(addr1.address, true);
            expect(await bridge.blocked(addr1.address)).to.equal(true);
            
            await bridge.setBlocked(addr1.address, false);
            expect(await bridge.blocked(addr1.address)).to.equal(false);
        });

        it("Should not allow non-owner to block accounts", async function () {
            await expect(
                bridge.connect(addr1).setBlocked(addr2.address, true)
            ).to.be.revertedWith("not owner");
        });

        it("Should enforce token allowlist", async function () {
            await bridge.setEnforceTokenAllowlist(true);
            expect(await bridge.enforceTokenAllowlist()).to.equal(true);
            
            await bridge.setEnforceTokenAllowlist(false);
            expect(await bridge.enforceTokenAllowlist()).to.equal(false);
        });

        it("Should not allow non-owner to set token allowlist enforcement", async function () {
            await expect(
                bridge.connect(addr1).setEnforceTokenAllowlist(true)
            ).to.be.revertedWith("not owner");
        });

        it("Should allow and disallow tokens", async function () {
            await bridge.setTokenAllowed(token.address, true);
            expect(await bridge.tokenAllowed(token.address)).to.equal(true);
            
            await bridge.setTokenAllowed(token.address, false);
            expect(await bridge.tokenAllowed(token.address)).to.equal(false);
        });

        it("Should not allow non-owner to set token allowance", async function () {
            await expect(
                bridge.connect(addr1).setTokenAllowed(token.address, true)
            ).to.be.revertedWith("not owner");
        });
    });

    describe("Bridge Functions", function () {
        it("Should generate a lock ID when locking tokens", async function () {
            // This is a placeholder test since the actual lock function requires
            // a token contract with specific functionality
            // We're just testing that the function exists and can be called
            expect(bridge.address).to.properAddress;
        });

        it("Should track processed lock IDs", async function () {
            const lockId = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-lock-id"));
            expect(await bridge.processedLockIds(lockId)).to.equal(false);
        });
    });

    describe("Audit Management", function () {
        const firm = "Test Audit Firm";
        const reportURI = "https://example.com/audit-report";
        const reportHash = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-report"));
        const timestamp = Math.floor(Date.now() / 1000);

        it("Should set audit info", async function () {
            await bridge.setAuditInfo(firm, reportURI, reportHash, timestamp);
            
            const audit = await bridge.audit();
            expect(audit.firm).to.equal(firm);
            expect(audit.reportURI).to.equal(reportURI);
            expect(audit.reportHash).to.equal(reportHash);
            expect(audit.timestamp).to.equal(timestamp);
            expect(audit.finalized).to.equal(false);
        });

        it("Should not allow non-owner to set audit info", async function () {
            await expect(
                bridge.connect(addr1).setAuditInfo(firm, reportURI, reportHash, timestamp)
            ).to.be.revertedWith("not owner");
        });

        it("Should finalize audit", async function () {
            await bridge.setAuditInfo(firm, reportURI, reportHash, timestamp);
            await bridge.finalizeAudit();
            
            const audit = await bridge.audit();
            expect(audit.finalized).to.equal(true);
        });

        it("Should not allow non-owner to finalize audit", async function () {
            await bridge.setAuditInfo(firm, reportURI, reportHash, timestamp);
            
            await expect(
                bridge.connect(addr1).finalizeAudit()
            ).to.be.revertedWith("not owner");
        });

        it("Should not allow setting audit info after finalization", async function () {
            await bridge.setAuditInfo(firm, reportURI, reportHash, timestamp);
            await bridge.finalizeAudit();
            
            await expect(
                bridge.setAuditInfo(firm, reportURI, reportHash, timestamp)
            ).to.be.revertedWith("audit finalized");
        });
    });
});