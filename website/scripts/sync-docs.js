import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '../../');
const WEBSITE_CONTENT = path.join(ROOT_DIR, 'website/src/content/docs');

const filesToMigrate = [
  ['README.md', 'overview.md'],
  ['README.ru.md', 'ru/overview.md'],
  ['docs/INSTALLATION.md', 'installation.md'],
  ['docs/INSTALLATION.ru.md', 'ru/installation.md'],
  ['docs/CONFIGURATION.md', 'configuration.md'],
  ['docs/CONFIGURATION.ru.md', 'ru/configuration.md'],
  ['docs/USAGE.md', 'usage.md'],
  ['docs/USAGE.ru.md', 'ru/usage.md'],
  ['docs/SHELL_INTEGRATION.md', 'shell-integration.md'],
  ['docs/SHELL_INTEGRATION.ru.md', 'ru/shell-integration.md'],
  ['CONTRIBUTING.md', 'contributing.md'],
  ['CHANGELOG.md', 'changelog.md'],
  ['CHANGELOG.ru.md', 'ru/changelog.md'],
];

for (const [src, destRel] of filesToMigrate) {
  const srcPath = path.join(ROOT_DIR, src);
  if (!fs.existsSync(srcPath)) continue;

  let content = fs.readFileSync(srcPath, 'utf8');

  // Extract title from the first H1 (# Title)
  let title = 'Untitled';
  const h1Match = content.match(/^#\s+(.+)$/m);
  if (h1Match) {
    title = h1Match[1].trim();
    // Remove the H1 from content since Starlight adds it automatically based on frontmatter
    content = content.replace(h1Match[0], '');
  }

  // Remove language switchers and badge paragraphs
  // This matches <p align="center"> ... </p> specifically targeting the English | Русский switcher and badges
  content = content.replace(/<p align="center">\s*(?:<a[^>]*>)?(?:<img[^>]*>)?(?:<\/a>)?(?:(?:&nbsp;|\||\s)*)(?:<a[^>]*>)?(?:<img[^>]*>)?(?:<\/a>)?(?:\s|🇬🇧|🇷🇺|<b>English<\/b>|<a[^>]*>Русский<\/a>|\|)*<\/p>/g, '');
  content = content.replace(/<p align="center">\s*🇬🇧\s*<b>English<\/b>\s*\|\s*🇷🇺\s*<a[^>]*>Русский<\/a>\s*<\/p>/gi, '');
  
  // Remove CI badges and view counters
  content = content.replace(/\[!\[.*?\]\(.*?\)\]\(.*?\)\n?/g, '');
  content = content.replace(/!\[Views\]\(.*?\)\n?/g, '');

  // Fix internal links
  const linkReplacements = {
    'docs/INSTALLATION.md': 'installation',
    'docs/CONFIGURATION.md': 'configuration',
    'docs/USAGE.md': 'usage',
    'docs/SHELL_INTEGRATION.md': 'shell-integration',
    'INSTALLATION.md': 'installation',
    'CONFIGURATION.md': 'configuration',
    'USAGE.md': 'usage',
    'SHELL_INTEGRATION.md': 'shell-integration',
    'README.md': 'overview',
    'README.ru.md': 'overview',
  };

  for (const [oldLink, newLink] of Object.entries(linkReplacements)) {
    // Regex to match markdown links
    const regex = new RegExp(`\\]\\((?:\\.\\/)?${oldLink.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&')}(#[^)]*)?\\)`, 'g');
    content = content.replace(regex, `](${newLink}$1)`);
  }

  // Rewrite image assets paths to point to src/assets
  // Find the depth of destRel (e.g. 'overview.md' -> depth 0 -> '../../', 'ru/overview.md' -> depth 1 -> '../../../')
  const depth = destRel.split('/').length - 1;
  const prefix = '../'.repeat(depth + 2); // content/docs -> src/assets requires ../../
  
  content = content.replace(/\]\(assets\//g, `](${prefix}assets/`);

  const frontmatter = `---\ntitle: ${title}\n---\n\n`;
  const newContent = frontmatter + content.trimStart();

  const destPath = path.join(WEBSITE_CONTENT, destRel);
  fs.mkdirSync(path.dirname(destPath), { recursive: true });
  fs.writeFileSync(destPath, newContent, 'utf8');
  console.log(`Synced ${src} to ${destRel}`);
}
