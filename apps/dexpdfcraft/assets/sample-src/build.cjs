// Prints sample.html to ../sample.pdf with Playwright's Chromium: node assets/sample-src/build.cjs
const path = require('path');
const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto('file://' + path.join(__dirname, 'sample.html'));
  await page.pdf({ path: path.join(__dirname, '..', 'sample.pdf'), format: 'A4', printBackground: true, preferCSSPageSize: true });
  await browser.close();
})();
