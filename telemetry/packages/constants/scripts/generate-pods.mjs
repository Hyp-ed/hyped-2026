import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Find config directory more reliably - look for it starting from script location
function findConfigDir() {
    let currentDir = __dirname;

    // Go up until we find a directory containing 'config'
    for (let i = 0; i < 5; i++) {
        const configPath = path.join(currentDir, 'config', 'pods.yaml');
        if (fs.existsSync(configPath)) {
        return configPath;
        }
        currentDir = path.join(currentDir, '..');
    }

    throw new Error('Could not find config/pods.yaml');
}
  
const configPath = findConfigDir();
console.log(`Reading config from: ${configPath}`);
  
const yamlContent = fs.readFileSync(configPath, 'utf8');

const output = `// Auto-generated - DO NOT EDIT
// Generated from config/pods.yaml
export const podsYamlContent = ${JSON.stringify(yamlContent)};
`;

const outputPath = path.join(__dirname, '..', 'src', 'pods', 'pods-data.generated.ts');
fs.writeFileSync(outputPath, output);

console.log('✓ Generated pods-data.generated.ts');