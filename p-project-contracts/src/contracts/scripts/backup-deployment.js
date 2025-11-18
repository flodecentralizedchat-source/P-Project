// backup-deployment.js - Script to backup deployment data
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

async function main() {
    console.log("P-Project Deployment Backup");
    console.log("==========================\n");
    
    try {
        // Create backup directory
        const backupDir = path.join(process.cwd(), "backups");
        if (!fs.existsSync(backupDir)) {
            fs.mkdirSync(backupDir);
            console.log("✅ Created backups directory");
        }
        
        // Generate timestamp for backup
        const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
        const backupSubDir = path.join(backupDir, `deployment-${timestamp}`);
        fs.mkdirSync(backupSubDir);
        console.log(`✅ Created backup directory: ${path.relative(process.cwd(), backupSubDir)}`);
        
        // Files to backup
        const backupFiles = [
            "deployment-info.json",
            "deployment-report.json",
            "verified-contracts.json",
            "initialized-contracts.json",
            "dex-prepared.json",
            ".env"
        ];
        
        console.log("\n1. Backing up deployment files...");
        
        let filesBackedUp = 0;
        for (const file of backupFiles) {
            const filePath = path.join(process.cwd(), file);
            if (fs.existsSync(filePath)) {
                const backupPath = path.join(backupSubDir, file);
                fs.copyFileSync(filePath, backupPath);
                console.log(`   ✅ Backed up ${file}`);
                filesBackedUp++;
            } else {
                console.log(`   ⚠️  ${file} not found, skipping`);
            }
        }
        
        // Backup directories
        const backupDirs = ["artifacts"];
        
        console.log("\n2. Backing up directories...");
        
        for (const dir of backupDirs) {
            const dirPath = path.join(process.cwd(), dir);
            if (fs.existsSync(dirPath)) {
                const backupPath = path.join(backupSubDir, dir);
                execSync(`cp -r "${dirPath}" "${backupPath}"`, { stdio: "pipe" });
                console.log(`   ✅ Backed up ${dir} directory`);
            } else {
                console.log(`   ⚠️  ${dir} directory not found, skipping`);
            }
        }
        
        // Create backup info file
        const backupInfo = {
            timestamp: new Date().toISOString(),
            filesBackedUp: filesBackedUp,
            backupPath: backupSubDir,
            nodeVersion: process.version,
            platform: process.platform
        };
        
        const backupInfoPath = path.join(backupSubDir, "backup-info.json");
        fs.writeFileSync(backupInfoPath, JSON.stringify(backupInfo, null, 2));
        console.log(`   ✅ Created backup info file`);
        
        console.log("\n🎉 Deployment backup completed!");
        console.log(`Backup location: ${path.relative(process.cwd(), backupSubDir)}`);
        console.log(`Files backed up: ${filesBackedUp}`);
        
        console.log("\nTo restore from this backup:");
        console.log(`1. Copy files from ${path.relative(process.cwd(), backupSubDir)} to project root`);
        console.log("2. Or use the restore script (if available)");
        
    } catch (error) {
        console.error("❌ Error creating backup:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });