// generate-report.js - Script to generate a deployment report
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("P-Project Deployment Report Generator");
    console.log("====================================\n");
    
    // Check for deployment info file
    const deploymentInfoFile = path.join(process.cwd(), "deployment-info.json");
    
    if (!fs.existsSync(deploymentInfoFile)) {
        console.log("❌ No deployment found. Run deployment first.");
        console.log("\nTo deploy:");
        console.log("1. npm run deploy:testnet  (for testnet)");
        console.log("2. npm run deploy:mainnet  (for mainnet)");
        return;
    }
    
    try {
        const deploymentInfo = JSON.parse(fs.readFileSync(deploymentInfoFile, "utf8"));
        
        // Generate report
        const report = {
            title: "P-Project Token Deployment Report",
            generated: new Date().toISOString(),
            network: deploymentInfo.network,
            deploymentTime: deploymentInfo.deploymentTime,
            deployer: deploymentInfo.deployer,
            contracts: deploymentInfo.contracts,
            status: {
                deployed: true,
                verified: fs.existsSync(path.join(process.cwd(), "verified-contracts.json")),
                initialized: fs.existsSync(path.join(process.cwd(), "initialized-contracts.json")),
                dexPrepared: fs.existsSync(path.join(process.cwd(), "dex-prepared.json"))
            }
        };
        
        // Save report to file
        const reportFile = path.join(process.cwd(), "deployment-report.json");
        fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
        
        console.log("✅ Deployment report generated successfully!");
        console.log("Report saved to:", reportFile);
        
        // Display summary
        console.log("\nDeployment Report Summary:");
        console.log("========================");
        console.log("Title:", report.title);
        console.log("Generated:", new Date(report.generated).toLocaleString());
        console.log("Network:", report.network);
        console.log("Deployment Time:", new Date(report.deploymentTime).toLocaleString());
        console.log("Deployer:", report.deployer);
        console.log("\nContracts:");
        for (const [contractName, address] of Object.entries(report.contracts)) {
            console.log(`  ${contractName}: ${address}`);
        }
        console.log("\nStatus:");
        console.log("  Deployed:", report.status.deployed ? "✅ Yes" : "❌ No");
        console.log("  Verified:", report.status.verified ? "✅ Yes" : "❌ No");
        console.log("  Initialized:", report.status.initialized ? "✅ Yes" : "❌ No");
        console.log("  DEX Prepared:", report.status.dexPrepared ? "✅ Yes" : "❌ No");
        
        if (report.status.verified && report.status.initialized && report.status.dexPrepared) {
            console.log("\n🎉 All deployment steps completed!");
            console.log("Your contracts are ready for DEX listing!");
        } else {
            console.log("\n⚠️  Some deployment steps are pending.");
            console.log("Check the status above and complete the remaining steps.");
        }
        
    } catch (error) {
        console.error("❌ Error generating deployment report:", error.message);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });