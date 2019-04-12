//! Shader compilation.

#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

#[cfg(feature = "spirv-reflection")]
pub mod reflect;
#[cfg(feature = "shader-compiler")]
mod shaderc;

#[cfg(feature = "spirv-reflection")]
pub use self::reflect::*;
#[cfg(feature = "shader-compiler")]
pub use self::shaderc::*;

/// Interface to create shader modules from shaders.
/// Implemented for static shaders via [`compile_to_spirv!`] macro.
///
pub trait Shader {
    /// Get spirv bytecode.
    fn spirv(&self) -> Result<std::borrow::Cow<'_, [u8]>, failure::Error>;

    #[cfg(feature = "spirv-reflection")]
    /// Uses spirv-reflect to generate a [SpirvShaderDescription] reflection representation, which is
    /// an intermediate to gfx_hal data representations.
    fn reflect(&self) -> Result<&SpirvShaderDescription, failure::Error> { unimplemented!("Shader reflection not implemented for this type") }

    /// Create shader module.
    ///
    /// Spir-V bytecode must adhere valid usage on this Vulkan spec page:
    /// https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkShaderModuleCreateInfo.html
    unsafe fn module<B>(
        &self,
        factory: &rendy_factory::Factory<B>,
    ) -> Result<B::ShaderModule, failure::Error>
    where
        B: gfx_hal::Backend,
    {
        gfx_hal::Device::create_shader_module(factory.device().raw(), &self.spirv()?)
            .map_err(Into::into)
    }
}

/// Spir-V shader.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpirvShader {
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    spirv: Vec<u8>,
}

impl SpirvShader {
    /// Create Spir-V shader from bytes.
    pub fn new(spirv: Vec<u8>) -> Self {
        assert!(!spirv.is_empty());
        assert_eq!(spirv.len() % 4, 0);
        Self { spirv }
    }
}

impl Shader for SpirvShader {
    fn spirv(&self) -> Result<std::borrow::Cow<'_, [u8]>, failure::Error> {
        Ok(std::borrow::Cow::Borrowed(&self.spirv))
    }
}


#[cfg(feature = "spirv-reflection")]
/// Spir-V shader with reflection
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SpirvReflectedShader {
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    spirv: Vec<u8>,
    reflection: SpirvShaderDescription,
}

#[cfg(feature = "spirv-reflection")]
impl SpirvReflectedShader {
    /// Create Spir-V shader from bytes.
    pub fn new(spirv: Vec<u8>) -> Self {
        assert!(!spirv.is_empty());
        assert_eq!(spirv.len() % 4, 0);
        let reflection = reflect::SpirvShaderDescription::from_bytes(spirv.as_slice()).unwrap();

        Self {
            spirv,
            reflection,
        }
    }
}

#[cfg(feature = "spirv-reflection")]
impl Shader for SpirvReflectedShader {
    fn spirv(&self) -> Result<std::borrow::Cow<'_, [u8]>, failure::Error> {
        Ok(std::borrow::Cow::Borrowed(&self.spirv))
    }

    fn reflect(&self) -> Result<&SpirvShaderDescription, failure::Error> {
        Ok(&self.reflection)
    }
}
