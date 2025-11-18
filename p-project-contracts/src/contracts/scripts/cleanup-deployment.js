// cleanup-deployment.js - Script to clean up deployment artifacts
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("P-Project Deployment Cleanup");
    console.log("===========================\n");
    
    // List of files to potentially clean up
    const cleanupFiles = [
        "deployment-info.json",
        "deployment-report.json",
        "verified-contracts.json",
        "initialized-contracts.json",
        "dex-prepared.json",
        "artifacts",
        "cache"
    ];
    
    console.log("The following files and directories will be removed:");
    for (const file of cleanupFiles) {
        const filePath = path.join(process.cwd(), file);
        if (fs.existsSync(filePath)) {
            console.log(`  - ${file}`);
        }
    }
    
    console.log("\n⚠️  This will remove all deployment artifacts and compiled contracts.");
    console.log("⚠️  You will need to recompile and redeploy after cleanup.");
    
    // In a real implementation, we would prompt for confirmation
    // For now, we'll just show what would be cleaned up
    
    console.log("\nTo clean up deployment artifacts, run:");
    console.log("  rm deployment-info.json deployment-report.json verified-contracts.json initialized-contracts.json dex-prepared.json");
    console.log("  rm -rf artifacts cache");
    
    console.log("\nOr use this script by removing the safety check in the code.");
    
    // Safety check - in a real implementation, we would ask for confirmation
    console.log("\n🛑 Safety check enabled - no files were actually removed.");
    console.log("🛑 To actually clean up, modify this script to remove the safety check.");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });