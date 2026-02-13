/**
 * SVG zu PNG Konvertierungs-Script
 *
 * Führe dieses Script aus mit: node scripts/convert-icons.js
 *
 * Voraussetzungen:
 * - npm install sharp
 *
 * Alternativ kannst du die SVG-Dateien manuell konvertieren:
 * - Öffne https://svgtopng.com/ oder https://cloudconvert.com/svg-to-png
 * - Lade die SVG-Dateien hoch
 * - Konvertiere zu PNG mit den angegebenen Größen
 */

const fs = require('fs');
const path = require('path');

// Versuche sharp zu importieren (optional)
let sharp;
try {
    sharp = require('sharp');
} catch (e) {
    console.log('⚠️  Sharp ist nicht installiert. Bitte installiere es mit: npm install sharp');
    console.log('   Oder konvertiere die SVGs manuell mit einem Online-Tool.\n');
    console.log('📁 SVG-Dateien befinden sich in: assets/images/');
    console.log('\n📝 Manuelle Konvertierung:');
    console.log('   1. icon.svg → icon.png (1024x1024)');
    console.log('   2. adaptive-icon.svg → adaptive-icon.png (1024x1024)');
    console.log('   3. splash-icon.svg → splash-icon.png (512x512)');
    console.log('   4. logo.svg → favicon.png (64x64)');
    process.exit(0);
}

const assetsDir = path.join(__dirname, '..', 'assets', 'images');

const conversions = [
    { input: 'icon.svg', output: 'icon.png', size: 1024 },
    { input: 'adaptive-icon.svg', output: 'adaptive-icon.png', size: 1024 },
    { input: 'splash-icon.svg', output: 'splash-icon.png', size: 512 },
    { input: 'logo.svg', output: 'favicon.png', size: 64 },
];

async function convertSvgToPng() {
    console.log('🎨 Konvertiere SVG-Dateien zu PNG...\n');

    for (const conversion of conversions) {
        const inputPath = path.join(assetsDir, conversion.input);
        const outputPath = path.join(assetsDir, conversion.output);

        if (!fs.existsSync(inputPath)) {
            console.log(`⚠️  ${conversion.input} nicht gefunden, überspringe...`);
            continue;
        }

        try {
            await sharp(inputPath)
                .resize(conversion.size, conversion.size)
                .png()
                .toFile(outputPath);

            console.log(`✅ ${conversion.input} → ${conversion.output} (${conversion.size}x${conversion.size})`);
        } catch (error) {
            console.error(`❌ Fehler bei ${conversion.input}:`, error.message);
        }
    }

    console.log('\n🎉 Fertig! Die Icons wurden konvertiert.');
}

convertSvgToPng();

