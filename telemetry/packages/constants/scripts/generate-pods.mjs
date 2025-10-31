import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const configPath = path.join(__dirname, '..', '..', '..', 'config', 'pods.yaml');
const yamlContent = fs.readFileSync(configPath, 'utf8');

const output = `// Auto-generated - DO NOT EDIT
// Generated from config/pods.yaml
export const podsYamlContent = ${JSON.stringify(yamlContent)};
`;

const outputPath = path.join(__dirname, '..', 'src', 'pods', 'pods-data.generated.ts');
fs.writeFileSync(outputPath, output);

console.log('✓ Generated pods-data.generated.ts');
