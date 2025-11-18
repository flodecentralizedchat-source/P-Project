// restore-deployment.js - Script to restore deployment data from backup
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

async function main() {
    console.log("P-Project Deployment Restore");
    console.log("==========================\n");
    
    try {
        // Check for backups directory
        const backupDir = path.join(process.cwd(), "backups");
        if (!fs.existsSync(backupDir)) {
            console.log("❌ No backups directory found.");
            console.log("   Create a backup first using: npm run backup:deployment");
            return;
        }
        
        // List available backups
        const backupDirs = fs.readdirSync(backupDir)
            .filter(file => fs.statSync(path.join(backupDir, file)).isDirectory())
            .sort((a, b) => b.localeCompare(a)); // Sort by name (newest first)
        
        if (backupDirs.length === 0) {
            console.log("❌ No backups found in the backups directory.");
            console.log("   Create a backup first using: npm run backup:deployment");
            return;
        }
        
        console.log("Available backups:");
        backupDirs.forEach((backup, index) => {
            console.log(`  ${index + 1}. ${backup}`);
        });
        
        console.log("\n⚠️  This script will restore deployment data from a backup.");
        console.log("⚠️  This will overwrite existing files in the project root.");
        console.log("⚠️  Make sure to backup current data if needed before proceeding.");
        
        // In a real implementation, we would prompt for selection
        // For now, we'll just show how to use it with a specific backup
        
        console.log("\nTo restore from a specific backup, run:");
        console.log("  node scripts/restore-deployment.js <backup-directory-name>");
        
        console.log("\nExample:");
        console.log(`  node scripts/restore-deployment.js ${backupDirs[0]}`);
        
        // Check if a backup name was provided as argument
        const args = process.argv.slice(2);
        if (args.length > 0) {
            const backupName = args[0];
            const backupPath = path.join(backupDir, backupName);
            
            if (!fs.existsSync(backupPath)) {
                console.log(`\n❌ Backup '${backupName}' not found.`);
                return;
            }
            
            console.log(`\nRestoring from backup: ${backupName}`);
            
            // Files to restore
            const restoreFiles = [
                "deployment-info.json",
                "deployment-report.json",
                "verified-contracts.json",
                "initialized-contracts.json",
                "dex-prepared.json",
                ".env"
            ];
            
            console.log("\n1. Restoring deployment files...");
            
            let filesRestored = 0;
            for (const file of restoreFiles) {
                const backupFilePath = path.join(backupPath, file);
                if (fs.existsSync(backupFilePath)) {
                    const projectFilePath = path.join(process.cwd(), file);
                    fs.copyFileSync(backupFilePath, projectFilePath);
                    console.log(`   ✅ Restored ${file}`);
                    filesRestored++;
                } else {
                    console.log(`   ⚠️  ${file} not found in backup, skipping`);
                }
            }
            
            // Restore directories
            const restoreDirs = ["artifacts"];
            
            console.log("\n2. Restoring directories...");
            
            for (const dir of restoreDirs) {
                const backupDirPath = path.join(backupPath, dir);
                if (fs.existsSync(backupDirPath)) {
                    const projectDirPath = path.join(process.cwd(), dir);
                    // Remove existing directory if it exists
                    if (fs.existsSync(projectDirPath)) {
                        fs.rmSync(projectDirPath, { recursive: true, force: true });
                    }
                    execSync(`cp -r "${backupDirPath}" "${projectDirPath}"`, { stdio: "pipe" });
                    console.log(`   ✅ Restored ${dir} directory`);
                } else {
                    console.log(`   ⚠️  ${dir} directory not found in backup, skipping`);
                }
            }
            
            console.log("\n🎉 Deployment restore completed!");
            console.log(`Files restored: ${filesRestored}`);
            
            // Read backup info if available
            const backupInfoPath = path.join(backupPath, "backup-info.json");
            if (fs.existsSync(backupInfoPath)) {
                const backupInfo = JSON.parse(fs.readFileSync(backupInfoPath, "utf8"));
                console.log(`\nBackup information:`);
                console.log(`  Created: ${new Date(backupInfo.timestamp).toLocaleString()}`);
                console.log(`  Files in backup: ${backupInfo.filesBackedUp}`);
                console.log(`  Node version: ${backupInfo.nodeVersion}`);
                console.log(`  Platform: ${backupInfo.platform}`);
            }
        }
        
    } catch (error) {
        console.error("❌ Error restoring backup:", error.message);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });