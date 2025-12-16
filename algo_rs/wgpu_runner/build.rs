use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("shader", "spirv-unknown-vulkan1.4")
        .capability(Capability::Int64)
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}
