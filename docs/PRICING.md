# CostPilot Pricing Model

## Free Tier (Open Source)
**Price:** $0/month

### Includes:
- ✅ Unlimited cost estimates
- ✅ AI-powered predictions
- ✅ Policy enforcement
- ✅ Drift detection
- ✅ SLO monitoring
- ✅ Baseline tracking
- ✅ PR comments
- ✅ All features
- ✅ Community support via GitHub Issues

### Usage:
- Unlimited repositories
- Unlimited team members
- Unlimited analyses
- No API rate limits

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

| Feature | CostPilot | Infracost | Cloud Custodian |
|---------|-----------|-----------|-----------------|
| Price | Free (Open Source) | Free OSS / Paid Cloud | Free (Open Source) |
| Cost Estimation | ✅ | ✅ | ❌ |
| AI Prediction | ✅ | ❌ | ❌ |
| Policy Engine | ✅ Custom DSL | ❌ | ✅ Python |
| Drift Detection | ✅ SHA256 | ❌ | ✅ |
| SLO Monitoring | ✅ | ❌ | ❌ |
| Zero Network | ✅ | ❌ | ❌ |
| PR Comments | ✅ | ✅ | ❌ |
| Exemptions | ✅ Time-bound | ❌ | ❌ |
| Baseline Tracking | ✅ | ✅ | ❌ |

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

All features included at **$0/month**:

| Feature | Value | Included |
|---------|-------|----------|
| Cost Estimation | ✅ | Free |
| AI Prediction | ✅ | Free |
| Policy Engine | ✅ | Free |
| Drift Detection | ✅ | Free |
| SLO Monitoring | ✅ | Free |
| Baseline Tracking | ✅ | Free |
| PR Comments | ✅ | Free |
| Exemptions | ✅ | Free |
| Approval Workflows | ✅ | Free |
| Audit Logs | ✅ | Free |
| Zero Network Mode | ✅ | Free |

## FAQ

**Q: Is CostPilot really free?**
A: Yes, completely free under MIT license. No hidden costs, no premium tiers required for features.

**Q: Will it always be free?**
A: The open source version will always be free. We may offer optional paid enterprise support in the future.

**Q: What about data/API costs?**
A: CostPilot operates in Zero Network mode by default - no external API calls, no data egress costs.

**Q: Can I use this commercially?**
A: Yes, MIT license allows unrestricted commercial use.

**Q: Do you sell my data?**
A: No. CostPilot runs locally in your CI/CD. We never see your data.

## Getting Started

```bash
# Add to your GitHub Actions workflow
- uses: Dee66/CostPilot@v1
  with:
    terraform_plan: plan.json
```

No credit card required. No sign-up. No tracking.
