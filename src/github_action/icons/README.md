# CostPilot Icon Assets

This directory contains icon assets for the CostPilot GitHub Action marketplace listing.

## Icon Specifications

### GitHub Actions Marketplace Requirements

- **Format:** PNG with transparency
- **Size:** 128x128 pixels (displayed at 64x64)
- **Color:** Match branding color #00D26A (green)
- **Background:** Transparent
- **Theme:** Should work on both light and dark backgrounds

### Icon Design

**Primary Icon:** Dollar sign ($) with circuit/network pattern
- Represents cost (dollar) + technology (circuits)
- Green color (#00D26A) for positive/savings theme
- Clean, modern, minimalist design
- Recognizable at small sizes

## Files

### icon.png
- 128x128 PNG with transparency
- Used in GitHub Actions Marketplace
- Primary branding asset

### icon.svg
- Vector source file
- Scalable for future use
- Editable design

### icon-light.png
- Optimized for light backgrounds
- Higher contrast version

### icon-dark.png
- Optimized for dark backgrounds
- Alternative color scheme if needed

## Brand Colors

- **Primary Green:** #00D26A
- **Dark Green:** #00A855
- **Light Green:** #66E5A8
- **Accent:** #0066FF
- **Text Dark:** #1A1A1A
- **Text Light:** #FFFFFF

## Usage

### GitHub Action Metadata

In `action.yml`:
```yaml
branding:
  icon: 'dollar-sign'
  color: 'green'
```

### Marketplace Listing

Upload `icon.png` (128x128) when submitting to marketplace.

### README Badges

```markdown
![CostPilot](icon.png)
```

## Design Guidelines

1. **Simple & Clear** - Recognizable at 16x16 pixels
2. **Brand Consistent** - Use official colors
3. **Technology Focus** - Represent FinOps/DevOps
4. **Professional** - Enterprise-ready appearance
5. **Unique** - Distinguishable from similar tools

## Creating the Icon

### Using Figma/Sketch

1. Create 128x128 artboard
2. Design with 16px grid
3. Use brand colors
4. Export as PNG with transparency
5. Optimize with ImageOptim or TinyPNG

### SVG to PNG Conversion

```bash
# Using ImageMagick
convert -background none -resize 128x128 icon.svg icon.png

# Using Inkscape
inkscape --export-type=png --export-width=128 --export-height=128 icon.svg -o icon.png
```

## Icon Concept

```
  ┌─────────────┐
  │   ╭─────╮   │
  │   │  $  │   │  ← Dollar sign
  │   ╰──┬──╯   │
  │   ╭──┴──╮   │
  │  ╱ ╲   ╱ ╲  │  ← Circuit/network pattern
  │ ●───●───●  │  ← Connection nodes
  │  ╲ ╱   ╲ ╱  │
  │   ╰─────╯   │
  └─────────────┘
```

## Alternative Icon Ideas

1. **Graph with Dollar Sign**
   - Upward trending graph
   - Dollar symbol integrated
   - Represents cost optimization

2. **Shield with Dollar**
   - Protection/governance theme
   - Dollar in center
   - FinOps security focus

3. **Gauge/Meter**
   - Cost meter/dashboard
   - Green zone indicator
   - Monitoring theme

4. **Coins with Code Brackets**
   - Coins/money symbol
   - Code brackets < >
   - IaC + cost fusion

## Testing

Test icon visibility:
- [ ] Light background (GitHub light mode)
- [ ] Dark background (GitHub dark mode)
- [ ] 16x16 thumbnail size
- [ ] 32x32 medium size
- [ ] 64x64 standard size
- [ ] 128x128 full size
- [ ] Browser favicon
- [ ] Mobile display

## Legal

- Icon design is property of GuardSuite
- Licensed under MIT (same as project)
- Original creation, no third-party assets
- Safe for commercial use

## Future Assets

- [ ] Social media icons (Twitter, LinkedIn)
- [ ] Website favicon set (16x16, 32x32, 48x48)
- [ ] Apple Touch icon (180x180)
- [ ] Android icon (192x192)
- [ ] Open Graph image (1200x630)
- [ ] Documentation header image
