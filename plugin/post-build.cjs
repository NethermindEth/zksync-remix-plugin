const fs = require("fs");
const cp = require("child_process");
const path = require('path')

const packageJson = require("./package.json");

const commitHash = cp
  .execSync("git rev-parse --short HEAD")
  .toString()
  .replace("\n", "");

const metaJson = path.join(process.cwd(), 'public', 'meta.json')

const meta = {
  version: packageJson.version + "-" + commitHash,
};

const data = JSON.stringify(meta);

fs.writeFileSync(metaJson, data, { encoding: "utf8" });
console.log('postbuild: Wrote application metadata to "public/meta.json"');
console.log("postbuild:", data)