// Ouvre chaque page à plusieurs largeurs, la fait défiler jusqu'en bas, et échoue si
// la page peut défiler à l'horizontale, si une ressource manque ou si le navigateur
// signale une erreur. Marche aussi sur l'export statique servi sous un sous-chemin.
// Usage : node e2e/overflow.mjs http://localhost:4100
//         node e2e/overflow.mjs http://localhost:8000/ROUTIA
import { chromium } from 'playwright';

const base = (process.argv[2] ?? 'http://localhost:4100').replace(/\/$/, '');
const widths = [320, 360, 390, 430, 768, 1024, 1440];
const pages = ['/', '/app/'];
const browser = await chromium.launch();
let failures = 0;

for (const path of pages) {
  for (const width of widths) {
    const mobile = width < 768;
    const ctx = await browser.newContext({ viewport: { width, height: 800 }, isMobile: mobile, hasTouch: mobile });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
    // Une ressource manquante (feuille de style, bundle, icône) est une erreur.
    page.on('response', (r) => r.status() >= 400 && errors.push(`${r.status()} ${r.url()}`));
    await page.goto(base + path, { waitUntil: 'networkidle' });
    const height = await page.evaluate(() => document.documentElement.scrollHeight);
    for (let y = 0; y < height; y += 400) {
      await page.evaluate((y) => window.scrollTo(0, y), y);
      await page.waitForTimeout(40);
    }
    await page.waitForTimeout(500);
    const r = await page.evaluate(() => {
      window.scrollTo(400, window.scrollY);
      const de = document.documentElement;
      return { scrollX: window.scrollX, scrollWidth: de.scrollWidth, width: de.clientWidth };
    });
    const ok = r.scrollX === 0 && r.scrollWidth <= r.width && errors.length === 0;
    if (!ok) failures++;
    console.log(`${ok ? '✓' : '✗'} ${path} ${width}px (scrollWidth ${r.scrollWidth}, scrollX ${r.scrollX})${errors.length ? ' erreurs : ' + errors.join(' | ') : ''}`);
    await ctx.close();
  }
}

await browser.close();
process.exit(failures ? 1 : 0);
