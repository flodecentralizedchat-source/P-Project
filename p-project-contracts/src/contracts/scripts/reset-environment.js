// reset-environment.js - Script to reset the deployment environment
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

async function main() {
    console.log("P-Project Environment Reset");
    console.log("==========================\n");
    
    try {
        // Clean up deployment artifacts
        console.log("1. Cleaning up deployment artifacts...");
        
        const cleanupFiles = [
            "deployment-info.json",
            "deployment-report.json",
            "verified-contracts.json",
            "initialized-contracts.json",
            "dex-prepared.json"
        ];
        
        for (const file of cleanupFiles) {
            const filePath = path.join(process.cwd(), file);
            if (fs.existsSync(filePath)) {
                fs.unlinkSync(filePath);
                console.log(`   ✅ Removed ${file}`);
            }
        }
        
        // Clean up directories
        const cleanupDirs = ["artifacts", "cache"];
        
        for (const dir of cleanupDirs) {
            const dirPath = path.join(process.cwd(), dir);
            if (fs.existsSync(dirPath)) {
                fs.rmSync(dirPath, { recursive: true, force: true });
                console.log(`   ✅ Removed ${dir} directory`);
            }
        }
        
        console.log("   ✅ Deployment artifacts cleaned up\n");
        
        // Recompile contracts
        console.log("2. Recompiling contracts...");
        execSync("npx hardhat compile", { stdio: "inherit" });
        console.log("   ✅ Contracts recompiled\n");
        
        // Reset environment variables if needed
        console.log("3. Checking environment configuration...");
        const envFile = path.join(process.cwd(), ".env");
        const envExampleFile = path.join(process.cwd(), ".env.example");
        
        if (fs.existsSync(envFile)) {
            console.log("   ✅ .env file found");
        } else if (fs.existsSync(envExampleFile)) {
            console.log("   ⚠️  .env file not found, but .env.example is available");
            console.log("   ℹ️  Remember to configure your .env file with actual values");
        } else {
            console.log("   ❌ Neither .env nor .env.example file found");
            console.log("   ℹ️  You need to create a .env file with your configuration");
        }
        
        console.log("\n🎉 Environment reset completed!");
        console.log("\nNext steps:");
        console.log("1. Configure your .env file with actual values if needed");
        console.log("2. Run 'npm run test:deployment' to test deployment scripts");
        console.log("3. Deploy to testnet with 'npm run deploy:testnet'");
        
    } catch (error) {
        console.error("❌ Error resetting environment:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });