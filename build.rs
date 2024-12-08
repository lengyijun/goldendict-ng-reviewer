use shadow_rs::BuildPattern;
use shadow_rs::ShadowBuilder;

fn main() {
    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build()
        .unwrap();
}
