// Rasterize the SVG art: hero whale (visual check) + 1024px app icon.
import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync, existsSync } from 'fs';

const render = (svgPath, pngPath, width) => {
  if (!existsSync(svgPath)) return console.log(`skip ${svgPath}`);
  const svg = readFileSync(svgPath, 'utf8');
  const png = new Resvg(svg, { fitTo: { mode: 'width', value: width } }).render().asPng();
  writeFileSync(pngPath, png);
  console.log(`${pngPath} ✓`);
};

render('assets/whale.svg', process.env.WHALE_OUT ?? 'assets/whale-check.png', 1120);
render('assets/icon.svg', 'assets/icon-1024.png', 1024);
