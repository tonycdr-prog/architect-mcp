const { copyFileSync, existsSync, rmSync } = require("node:fs");

const packagePath = "package.json";
const backupPath = "package.json.prepack-backup";

if (existsSync(backupPath)) {
  copyFileSync(backupPath, packagePath);
  rmSync(backupPath);
}
