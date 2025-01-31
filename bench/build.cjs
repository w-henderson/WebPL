const fs = require("fs");
const path = require("path");

const dir = path.join(__dirname, "suite");
const outDir = path.join(__dirname, "dist");

let output = "export default {\n";

fs.readdirSync(dir).forEach(file => {
  const filePath = path.join(dir, file);
  const fileContent = fs.readFileSync(filePath, "utf-8");
  const benchmarkName = path.basename(file, ".pl");

  output += `  // Path: bench/suite/${benchmarkName}.pl\n`;
  output += `  "${benchmarkName}": ${JSON.stringify(fileContent)},\n`;
});

output += "};\n";

if (!fs.existsSync(outDir)) {
  fs.mkdirSync(outDir);
}
fs.writeFileSync(path.join(outDir, "benchmarks.js"), output);