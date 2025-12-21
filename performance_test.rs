use std::time::Instant;
use costpilot::engines::detection::DetectionEngine;

fn main() {
    println!("Testing CostPilot performance with large plans...");

    // Test with 1000 resources first
    let engine = DetectionEngine::new();
    let mut large_plan = r#"{"planned_values": {"root_module": {"resources": ["#.to_string();

    for i in 0..1000 {
        if i > 0 { large_plan.push(','); }
        large_plan.push_str(&format!(r#"{{"address": "test{}","values": {{"instance_type": "t2.micro"}}}}"#, i));
    }
    large_plan.push_str(r#"]}}}"#);

    println!("Plan size: {} bytes", large_plan.len());

    let start = Instant::now();
    let result = engine.detect_from_terraform_json(&large_plan);
    let duration = start.elapsed();

    match result {
        Ok(resources) => {
            println!("✅ Successfully processed {} resources in {:?}", resources.len(), duration);
            println!("  Average time per resource: {:?}", duration / resources.len() as u32);

            // Extrapolate to 300MB file
            // 300MB = 300 * 1024 * 1024 = 314,572,800 bytes
            // Current plan is ~50KB for 1000 resources, so ~50 bytes per resource
            let bytes_per_resource = large_plan.len() as f64 / 1000.0;
            let estimated_resources_300mb = (300.0 * 1024.0 * 1024.0) / bytes_per_resource;
            let estimated_time_300mb = duration.mul_f64(estimated_resources_300mb / 1000.0);

            println!("\n📊 Extrapolation for 300MB file:");
            println!("  Estimated resources: {:.0}", estimated_resources_300mb);
            println!("  Estimated time: {:?}", estimated_time_300mb);
            println!("  Bytes per resource: {:.1}", bytes_per_resource);
        }
        Err(e) => {
            println!("❌ Failed to process plan: {:?}", e);
        }
    }
}