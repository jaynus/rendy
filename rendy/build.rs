use rendy_shader::{
    SpirvShader, StaticShaderInfo, SpirvReflection, ShaderKind, SourceLanguage, ShaderSetBuilder,
    PodGenerator
};

#[cfg(feature = "pod-codegen")]
fn main() {
    println!("ENTER ENTER");

    let VERTEX: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/meshes/shader.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    let FRAGMENT: SpirvShader = StaticShaderInfo::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/meshes/shader.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    let SHADER_SET: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&VERTEX).unwrap()
        .with_fragment(&FRAGMENT).unwrap()
        .reflect().unwrap();

    let pods = SHADER_SET.generate_pods();

    panic!("WTF Hello World!: pods={}", pods);
}