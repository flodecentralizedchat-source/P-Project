// check-environment.js - Script to verify deployment environment setup
const fs = require("fs");
const path = require("path");

async function main() {
    console.log("P-Project Environment Setup Checker");
    console.log("====================================\n");
    
    // Check if we're in the right directory
    const currentDir = process.cwd();
    console.log("Current directory:", currentDir);
    
    // Check for required files
    console.log("1. Checking required files...");
    
    const requiredFiles = [
        "package.json",
        "hardhat.config.js",
        "PProjectToken.sol",
        "Vesting.sol",
        "Treasury.sol",
        "LiquidityPool.sol",
        "deploy.js"
    ];
    
    let allFilesPresent = true;
    
    for (const file of requiredFiles) {
        const filePath = path.join(currentDir, file);
        if (fs.existsSync(filePath)) {
            console.log(`   ✅ ${file} found`);
        } else {
            console.log(`   ❌ ${file} missing`);
            allFilesPresent = false;
        }
    }
    
    if (!allFilesPresent) {
        console.log("\n❌ Some required files are missing. Please check your installation.");
        return;
    }
    
    console.log("   ✅ All required files present\n");
    
    // Check for scripts directory and scripts
    console.log("2. Checking scripts...");
    
    const scriptsDir = path.join(currentDir, "scripts");
    if (fs.existsSync(scriptsDir)) {
        console.log("   ✅ scripts directory found");
        
        const requiredScripts = [
            "verify-deployment.js",
            "initialize-contracts.js",
            "prepare-dex-listing.js",
            "deployment-checklist.js",
            "test-deployment.js"
        ];
        
        let allScriptsPresent = true;
        
        for (const script of requiredScripts) {
            const scriptPath = path.join(scriptsDir, script);
            if (fs.existsSync(scriptPath)) {
                console.log(`   ✅ ${script} found`);
            } else {
                console.log(`   ❌ ${script} missing`);
                allScriptsPresent = false;
            }
        }
        
        if (!allScriptsPresent) {
            console.log("\n❌ Some required scripts are missing.");
            return;
        }
        
        console.log("   ✅ All required scripts present\n");
    } else {
        console.log("   ❌ scripts directory missing");
        return;
    }
    
    // Check for environment file
    console.log("3. Checking environment configuration...");
    
    const envFile = path.join(currentDir, ".env");
    const envExampleFile = path.join(currentDir, ".env.example");
    
    if (fs.existsSync(envFile)) {
        console.log("   ✅ .env file found");
        console.log("   ⚠️  Remember to never commit this file to version control!\n");
    } else if (fs.existsSync(envExampleFile)) {
        console.log("   ⚠️  .env file not found, but .env.example is available");
        console.log("   ℹ️  Copy .env.example to .env and configure your values\n");
    } else {
        console.log("   ❌ Neither .env nor .env.example file found");
        console.log("   ℹ️  You need to create a .env file with your configuration\n");
    }
    
    // Check Node.js and npm versions
    console.log("4. Checking Node.js and npm versions...");
    
    try {
        const nodeVersion = process.version;
        console.log(`   ✅ Node.js version: ${nodeVersion}`);
        
        // Check if Node.js version is >= 14
        const nodeMajorVersion = parseInt(nodeVersion.split(".")[0].replace("v", ""));
        if (nodeMajorVersion >= 14) {
            console.log("   ✅ Node.js version is sufficient (>= 14)");
        } else {
            console.log("   ❌ Node.js version is too old (requires >= 14)");
        }
    } catch (error) {
        console.log("   ❌ Unable to determine Node.js version");
    }
    
    try {
        const { execSync } = require("child_process");
        const npmVersion = execSync("npm --version", { encoding: "utf-8" }).trim();
        console.log(`   ✅ npm version: ${npmVersion}`);
    } catch (error) {
        console.log("   ❌ Unable to determine npm version");
    }
    
    console.log("\n5. Checking npm dependencies...");
    
    try {
        const packageJson = JSON.parse(fs.readFileSync(path.join(currentDir, "package.json"), "utf8"));
        
        const requiredDependencies = [
            "hardhat",
            "ethers",
            "@nomiclabs/hardhat-ethers",
            "@nomiclabs/hardhat-waffle",
            "ethereum-waffle",
            "chai",
            "dotenv"
        ];
        
        let allDependenciesPresent = true;
        
        for (const dep of requiredDependencies) {
            if (packageJson.dependencies && packageJson.dependencies[dep]) {
                console.log(`   ✅ ${dep} found in dependencies`);
            } else if (packageJson.devDependencies && packageJson.devDependencies[dep]) {
                console.log(`   ✅ ${dep} found in devDependencies`);
            } else {
                console.log(`   ❌ ${dep} missing from dependencies`);
                allDependenciesPresent = false;
            }
        }
        
        if (allDependenciesPresent) {
            console.log("   ✅ All required dependencies present");
        } else {
            console.log("   ❌ Some required dependencies are missing");
            console.log("   ℹ️  Run 'npm install' to install dependencies");
        }
    } catch (error) {
        console.log("   ❌ Unable to check dependencies:", error.message);
    }
    
    console.log("\n🎉 Environment check completed!");
    console.log("\nNext steps:");
    console.log("1. If any checks failed, address the issues");
    console.log("2. Run 'npm install' to ensure all dependencies are installed");
    console.log("3. Run 'npm run test:deployment' to test deployment scripts");
    console.log("4. Deploy to testnet with 'npm run deploy:testnet'");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });