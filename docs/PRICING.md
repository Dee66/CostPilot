# CostPilot Pricing Model

## Free Tier (Community Edition)
**Price:** $0/month

### Includes:
- ✅ Unlimited cost estimates
- ✅ Basic cost predictions (static analysis only)
- ✅ Policy enforcement (basic rules)
- ✅ Community support via GitHub Issues

### Usage:
- Unlimited repositories
- Unlimited team members
- Unlimited analyses
- No API rate limits

### Limitations:
- No advanced heuristics or AI predictions
- No autofix or patch generation
- No drift detection or anomaly detection
- No SLO monitoring or enforcement
- No premium analytics features

## Premium Tier (Founding Engineer Edition)
**Price:** $29/month

### Includes Everything in Free, Plus:
- ✅ Advanced heuristics engine (encrypted, licensed)
- ✅ Autofix and patch safety engine
- ✅ Drift detection and anomaly detection
- ✅ SLO monitoring and enforcement
- ✅ Economic threat detection
- ✅ Multi-depth dependency mapping
- ✅ Sustainability Analytics (carbon footprint, energy efficiency, fairness testing, transparency validation, social impact assessment)
- ✅ WASM Pro engine
- ✅ License token support
- ✅ Marketplace installation options

### Unique Perks:
- **Sustainability Analytics**: Comprehensive ESG compliance testing that integrates with GuardSuite threat detection for sustainability risk analysis
- **Enterprise-Ready**: Advanced features for serious engineering teams and organizations
- **IP Protection**: Encrypted heuristics bundle and commercial protections

## License
MIT License - Free for personal and commercial use

## Support Options

### Community Support (Free)
- GitHub Issues
- GitHub Discussions
- Documentation
- Examples repository

### Enterprise Support (Contact Sales)
- Priority bug fixes
- Custom feature development
- SLA guarantees
- Dedicated support channel
- Training and onboarding
- Architecture consulting

## Comparison with Alternatives

| Feature | CostPilot Free | CostPilot Premium | Infracost | Cloud Custodian |
|---------|----------------|-------------------|-----------|-----------------|
| Price | Free (Open Source) | $29/month | Free OSS / Paid Cloud | Free (Open Source) |
| Cost Estimation | ✅ | ✅ | ✅ | ❌ |
| AI Prediction | ❌ | ✅ | ❌ | ❌ |
| Policy Engine | ✅ Basic | ✅ Custom DSL | ❌ | ✅ Python |
| Drift Detection | ❌ | ✅ SHA256 | ❌ | ✅ |
| SLO Monitoring | ❌ | ✅ | ❌ | ❌ |
| Sustainability Analytics | ❌ | ✅ | ❌ | ❌ |
| Zero Network | ✅ | ✅ | ❌ | ❌ |
| PR Comments | ❌ | Future ✅ | ✅ | ❌ |
| Exemptions | ❌ | ✅ Time-bound | ❌ | ❌ |
| Baseline Tracking | ❌ | ✅ | ✅ | ❌ |

## ROI Calculator

### Typical Savings
Organizations using CostPilot report:
- **15-30%** reduction in cloud costs
- **4-8 hours/week** saved on manual cost reviews
- **60%** reduction in cost regression incidents
- **80%** improvement in policy compliance

### Example ROI
**Company:** Mid-size SaaS company
**Cloud Spend:** $50,000/month

**Before CostPilot:**
- Monthly cloud waste: $7,500 (15%)
- Manual review time: 8 hours/week × $100/hour = $3,200/month
- Cost incidents: 5/month × 4 hours × $150/hour = $3,000/month
- **Total Monthly Cost of Inefficiency:** $13,700

**After CostPilot:**
- Cloud waste reduced: $5,000/month saved
- Automated reviews: $3,200/month saved
- Fewer incidents: $2,500/month saved
- **Total Monthly Savings:** $10,700

**Annual ROI:** $128,400
**Cost to Implement:** $0 (open source)
**Net Benefit:** $128,400/year

## GitHub Actions Minutes Consumption

CostPilot is highly efficient:
- **Typical run time:** 30-60 seconds
- **GitHub Actions cost:** ~$0.008 per run (Linux)
- **Monthly cost** (100 runs): ~$0.80

## Self-Hosted Runner Option

For maximum cost efficiency:
- Run on your own infrastructure
- Zero GitHub Actions minutes consumed
- Complete data privacy
- Unlimited usage

## Cost Breakdown by Feature

| Feature | Community Edition | Premium Edition |
|---------|-------------------|-----------------|
| Cost Estimation | ✅ Free | ✅ Free |
| Basic Predictions | ✅ Free | ✅ Free |
| Policy Engine (Basic) | ✅ Free | ✅ Free |
| Zero Network Mode | ✅ Free | ✅ Free |
| Advanced Heuristics | ❌ | ✅ $29/month |
| Autofix & Patches | ❌ | ✅ $29/month |
| Drift Detection | ❌ | ✅ $29/month |
| Anomaly Detection | ❌ | ✅ $29/month |
| SLO Monitoring | ❌ | ✅ $29/month |
| Sustainability Analytics | ❌ | ✅ $29/month |
| Economic Threat Detection | ❌ | ✅ $29/month |
| Multi-depth Mapping | ❌ | ✅ $29/month |
| PR Comments | ❌ | Future ✅ $29/month |
| Exemptions | ❌ | ✅ $29/month |
| Baseline Tracking | ❌ | ✅ $29/month |

## FAQ

**Q: Is CostPilot really free?**
A: Yes, the community edition is completely free under MIT license with powerful basic features. For advanced capabilities like AI predictions, autofix, and sustainability analytics, we offer a premium tier at $29/month.

**Q: What's included in the premium tier?**
A: The Founding Engineer Edition ($29/month) includes everything in free plus advanced heuristics, autofix, drift detection, anomaly detection, SLO monitoring, sustainability analytics, and enterprise-ready features with IP protection.

**Q: Will the free version always be available?**
A: Yes, the community edition will always remain free and open source. The premium tier provides additional advanced features for serious engineering teams.

**Q: What about data/API costs?**
A: CostPilot operates in Zero Network mode by default - no external API calls, no data egress costs.

**Q: Can I use this commercially?**
A: Yes, MIT license allows unrestricted commercial use for the free edition. Premium features require a license.

**Q: Do you sell my data?**
A: No. CostPilot runs locally in your CI/CD. We never see your data.

## License Installation

Premium features require a valid license file. After purchasing CostPilot Premium:

1. **Download your license file** from the customer portal or email attachment
2. **Install the license file** to `~/.costpilot/license.json`:
   ```bash
   mkdir -p ~/.costpilot
   cp /path/to/your/license.json ~/.costpilot/license.json
   ```
3. **Install the ProEngine bundle** (provided with your license):
   ```bash
   cp /path/to/pro-engine.wasm.enc ~/.costpilot/
   cp /path/to/pro-engine.sig ~/.costpilot/
   ```
4. **Verify installation**:
   ```bash
   costpilot --version  # Should show Premium features enabled
   ```

The license file is a JSON document containing your subscription details and cryptographic keys for decrypting the ProEngine WASM bundle.

## Getting Started

```bash
# Add to your GitHub Actions workflow
- uses: Dee66/CostPilot@v1
  with:
    terraform_plan: plan.json
```

**Community Edition:** No credit card required. No sign-up. No tracking.

**Premium Edition:** License token required for advanced features. Visit [pricing page] for details.
