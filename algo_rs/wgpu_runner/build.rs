use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("shader", "spirv-unknown-spv1.6")
        .capability(Capability::Int64)
        // .capability(Capability::VariablePointers)
        // .extra_arg("--no-spirv-opt")
        // .extra_arg("--no-compact-ids")
        //.extra_arg("--no-structurize")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}
