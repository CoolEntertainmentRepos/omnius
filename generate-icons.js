// Generate Omnius app icons
const fs = require('fs');
const { execSync } = require('child_process');

// The O logo SVG with background
const createIconSVG = (size) => `<?xml version="1.0" encoding="UTF-8"?>
<svg width="${size}" height="${size}" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect width="512" height="512" rx="0" fill="#141414"/>
  <g transform="translate(106, 106) scale(45)">
    <path d="M3.336 6.69596C2.696 6.69596 2.124 6.55196 1.62 6.26396C1.116 5.97596 0.72 5.57996 0.432 5.07596C0.144 4.57196 0 3.99996 0 3.35996C0 2.71196 0.144 2.13596 0.432 1.63196C0.72 1.12796 1.116 0.731963 1.62 0.443963C2.124 0.155963 2.696 0.0119629 3.336 0.0119629C3.976 0.0119629 4.544 0.155963 5.04 0.443963C5.544 0.731963 5.94 1.12796 6.228 1.63196C6.516 2.13596 6.664 2.71196 6.672 3.35996C6.672 3.99996 6.524 4.57196 6.228 5.07596C5.94 5.57996 5.544 5.97596 5.04 6.26396C4.544 6.55196 3.976 6.69596 3.336 6.69596ZM3.336 5.85596C3.8 5.85596 4.216 5.74796 4.584 5.53196C4.952 5.31596 5.24 5.01996 5.448 4.64396C5.656 4.26796 5.76 3.83996 5.76 3.35996C5.76 2.87996 5.656 2.45196 5.448 2.07596C5.24 1.69196 4.952 1.39196 4.584 1.17596C4.216 0.959963 3.8 0.851963 3.336 0.851963C2.872 0.851963 2.456 0.959963 2.088 1.17596C1.72 1.39196 1.428 1.69196 1.212 2.07596C1.004 2.45196 0.9 2.87996 0.9 3.35996C0.9 3.83996 1.004 4.26796 1.212 4.64396C1.428 5.01996 1.72 5.31596 2.088 5.53196C2.456 5.74796 2.872 5.85596 3.336 5.85596Z" fill="#FF2D55"/>
  </g>
</svg>`;

// Create SVG files
const sizes = [32, 128, 256, 512];
const iconDir = 'src-tauri/icons';

sizes.forEach(size => {
  const svg = createIconSVG(size);
  fs.writeFileSync(`${iconDir}/icon-${size}.svg`, svg);
  console.log(`Created icon-${size}.svg`);
});

// Main icon
fs.writeFileSync(`${iconDir}/icon.svg`, createIconSVG(512));
console.log('Created icon.svg');

console.log('\nNow run: npx @aspect-build/aspect-cli run tauri icon src-tauri/icons/icon.svg');
console.log('Or use: npx tauri icon src-tauri/icons/icon.svg');
