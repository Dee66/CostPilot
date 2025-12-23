# CostPilot Demo GIF Script (60 seconds)

## Storyboard

**Duration:** 60 seconds
**Resolution:** 1280x720 or 1920x1080
**Format:** Animated GIF or MP4
**Frame rate:** 30 fps (optimized to 15 fps for GIF)

---

### Scene 1: Opening (0-5s)
**Duration:** 5 seconds

```
┌─────────────────────────────────────┐
│                                     │
│         💰 CostPilot                │
│                                     │
│   AI-Powered Cost Analysis for     │
│    Infrastructure as Code          │
│                                     │
└─────────────────────────────────────┘
```

**Text:** "CostPilot - Know your costs before you deploy"

---

### Scene 2: Developer Makes Changes (5-12s)
**Duration:** 7 seconds

**Screen:** VS Code with Terraform file

```hcl
# main.tf
resource "aws_instance" "web" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t3.micro"     # ← cursor here

  tags = {
    Name = "web-server"
  }
}
```

**Action:**
1. Cursor moves to "t3.micro"
2. Changes to "t3.xlarge"
3. File saves (brief flash)
4. Terminal appears at bottom

**Terminal:**
```bash
$ terraform plan -out=plan.tfplan
$ terraform show -json plan.tfplan > plan.json
$ git add .
$ git commit -m "Upsize web server"
$ git push origin feature/scale-up
```

---

### Scene 3: PR Creation (12-18s)
**Duration:** 6 seconds

**Screen:** GitHub PR interface

```
Create Pull Request

Title: Upsize web server for traffic ↑
Branch: feature/scale-up → main

✓ Terraform plan attached
✓ CostPilot workflow triggered
```

**Action:**
1. Form fills in
2. "Create pull request" button click
3. PR page loads
4. Checks section shows: "CostPilot / cost-analysis" (yellow dot, "In progress")

---

### Scene 4: CostPilot Analyzing (18-28s)
**Duration:** 10 seconds

**Screen:** GitHub Actions workflow running

```
┌─ CostPilot Analysis ─────────────────┐
│                                      │
│ ✓ Setup Terraform                    │
│ ✓ Generate plan                      │
│ ⏳ Run CostPilot...                  │
│   • Parsing Terraform plan           │
│   • Calculating costs                │
│   • Checking policies                │
│   • Comparing baseline               │
│   • Generating report                │
│                                      │
│ [==========>      ] 60%              │
└──────────────────────────────────────┘
```

**Progress bar fills up**

---

### Scene 5: Results Comment (28-45s)
**Duration:** 17 seconds

**Screen:** PR comment from CostPilot bot

```markdown
## 💰 CostPilot Analysis

**Total Monthly Cost:** $125.50 (+$118.00)
**Cost Change:** 📈 +1,573% from baseline

### 📊 Cost Breakdown
| Resource | Type | Monthly Cost | Change |
|----------|------|--------------|--------|
| aws_instance.web | t3.xlarge | $125.50 | +$118.00 |

### ⚠️ Cost Regression Detected

**Baseline:** $7.50/month
**Current:** $125.50/month
**Increase:** $118.00 (+1,573%)

### 🛡️ Policy Check
❌ **Cost Increase Limit Exceeded**
- Policy: Max 50% cost increase
- Actual: 1,573% increase
- Action: **Build blocked**

### 💡 Recommendations
1. Consider t3.large instead (saves $62/month)
2. Use Reserved Instances (save 40%)
3. Enable auto-scaling for variable load
4. Request exemption if increase is justified

---
*Analysis completed in 12s*
```

**Action:** Slow scroll through comment showing all details

---

### Scene 6: Developer Response (45-52s)
**Duration:** 7 seconds

**Split screen:**

**Left:** Developer terminal
```bash
$ vim terraform/main.tf
# Changes t3.xlarge → t3.large
$ git commit -am "Use t3.large based on CostPilot"
$ git push
```

**Right:** PR checks update
```
✓ CostPilot / cost-analysis
  Total: $63.50 (+$56.00, +884%)
  Status: ✅ Approved
```

---

### Scene 7: Success & Features (52-60s)
**Duration:** 8 seconds

**Screen:** Feature highlights

```
┌────────────────────────────────────┐
│     CostPilot Features            │
│                                    │
│ ✅ AI Cost Predictions             │
│ ✅ Policy Enforcement              │
│ ✅ Drift Detection                 │
│ ✅ SLO Monitoring                  │
│ ✅ Baseline Tracking               │
│ ✅ PR Comments                     │
│                                    │
│    github.com/Dee66/CostPilot     │
│         MIT License • Free         │
└────────────────────────────────────┘
```

**Fade out with CTA:** "Try CostPilot Today →"

---

## Recording Instructions

### Tools
- **Screen Recording:** Asciinema or OBS Studio
- **Editing:** FFmpeg or Adobe After Effects
- **GIF Conversion:** Gifski or FFmpeg
- **Text Overlay:** Keynote or After Effects

### Settings
```bash
# Record terminal with asciinema
asciinema rec -t "CostPilot Demo" demo.cast

# Convert to GIF with gifski
gifski -o demo.gif --fps 15 --quality 90 --width 1280 frame*.png

# Or use FFmpeg
ffmpeg -i input.mp4 -vf "fps=15,scale=1280:-1:flags=lanczos" \
  -c:v gif -f gif demo.gif
```

### Optimization
- Max file size: 10 MB for GitHub
- Use palette optimization for GIF
- Reduce to 15 fps for file size
- Use WebP or MP4 for better quality

### Accessibility
- Include captions/subtitles
- High contrast colors
- Clear, readable fonts (16pt minimum)
- Pause points for reading

## Alternative Versions

### 30-Second Version
- Remove Scene 6 (developer response)
- Faster transitions
- Focus on core value proposition

### 15-Second Version
- Scene 1: Opening (2s)
- Scene 4: Analysis (6s)
- Scene 5: Results (5s)
- Scene 7: CTA (2s)

### Social Media Cuts
- **Twitter:** 30s, 1280x720, MP4
- **LinkedIn:** 60s, 1920x1080, MP4
- **Product Hunt:** GIF, optimized
- **README:** GIF, embedded

## Placement

- Repository README.md (top)
- GitHub Marketplace listing
- Product Hunt submission
- Twitter announcement
- LinkedIn post
- Documentation site
- Blog post header

## Call to Action

Final frame should direct to:
- `github.com/Dee66/CostPilot` (primary)
- "Star ⭐ if useful"
- "Try the Action →"
