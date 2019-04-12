//! Using spirv-reflect-rs for reflection.
//!

use crate::Shader;
use gfx_hal::format::Format;
use spirv_reflect::{types::*, ShaderModule};

/// Workaround extension trait copy of std::convert::From, for simple conversion from spirv-reflect types to gfx_hal types
pub trait ReflectInto<T>: Sized {
    /// Attempts to perform a conversion from the provided type into this type
    fn reflect_into(&self) -> Result<T, failure::Error> {
        Err(failure::format_err!("Unsupported conversion type"))
    }
}

impl ReflectInto<Format> for ReflectFormat {
    fn reflect_into(&self) -> Result<Format, failure::Error> {
        match self {
            ReflectFormat::Undefined => Err(failure::format_err!("Undefined Format")),
            ReflectFormat::R32_UINT => Ok(Format::R32Uint),
            ReflectFormat::R32_SINT => Ok(Format::R32Int),
            ReflectFormat::R32_SFLOAT => Ok(Format::R32Float),
            ReflectFormat::R32G32_UINT => Ok(Format::Rg32Uint),
            ReflectFormat::R32G32_SINT => Ok(Format::Rg32Int),
            ReflectFormat::R32G32_SFLOAT => Ok(Format::Rg32Float),
            ReflectFormat::R32G32B32_UINT => Ok(Format::Rgb32Uint),
            ReflectFormat::R32G32B32_SINT => Ok(Format::Rgb32Int),
            ReflectFormat::R32G32B32_SFLOAT => Ok(Format::Rgb32Float),
            ReflectFormat::R32G32B32A32_UINT => Ok(Format::Rgb32Uint),
            ReflectFormat::R32G32B32A32_SINT => Ok(Format::Rgb32Int),
            ReflectFormat::R32G32B32A32_SFLOAT => Ok(Format::Rgb32Float),
        }
    }
}

fn type_element_format(
    flags: ReflectTypeFlags,
    traits: &ReflectTypeDescriptionTraits,
) -> Result<Format, failure::Error> {
    enum NumTy {
        SInt,
        UInt,
        Float,
    }

    let num_ty = if flags.contains(ReflectTypeFlags::INT) {
        match traits.numeric.scalar.signedness {
            0 => NumTy::UInt,
            1 => NumTy::SInt,
            _ => {
                failure::bail!(
                    "Unrecognized numeric signedness {:?}",
                    traits.numeric.scalar.signedness
                );
            }
        }
    } else if flags.contains(ReflectTypeFlags::FLOAT) {
        NumTy::Float
    } else {
        failure::bail!("Unrecognized numeric type with flags {:?}", flags);
    };

    let current_type = match (num_ty, traits.numeric.scalar.width) {
        (NumTy::SInt, 8) => Format::R8Int,
        (NumTy::SInt, 16) => Format::R16Int,
        (NumTy::SInt, 32) => Format::R32Int,
        (NumTy::SInt, 64) => Format::R64Int,
        (NumTy::UInt, 8) => Format::R8Uint,
        (NumTy::UInt, 16) => Format::R16Uint,
        (NumTy::UInt, 32) => Format::R32Uint,
        (NumTy::UInt, 64) => Format::R64Uint,
        (NumTy::Float, 32) => Format::R32Float,
        (NumTy::Float, 64) => Format::R64Float,
        _ => {
            failure::bail!(
                "Unrecognized numeric type with width {}",
                traits.numeric.scalar.width
            );
        }
    };

    if traits.numeric.vector.component_count > 1 {
        Ok(
            match (current_type, traits.numeric.vector.component_count) {
                (Format::R8Int, 2) => Format::Rg8Int,
                (Format::R8Int, 3) => Format::Rgb8Int,
                (Format::R8Int, 4) => Format::Rgba8Int,
                (Format::R16Int, 2) => Format::Rg16Int,
                (Format::R16Int, 3) => Format::Rgb16Int,
                (Format::R16Int, 4) => Format::Rgba16Int,
                (Format::R32Int, 2) => Format::Rg32Int,
                (Format::R32Int, 3) => Format::Rgb32Int,
                (Format::R32Int, 4) => Format::Rgba32Int,
                (Format::R64Int, 2) => Format::Rg64Int,
                (Format::R64Int, 3) => Format::Rgb64Int,
                (Format::R64Int, 4) => Format::Rgba64Int,
                (Format::R8Uint, 2) => Format::Rg8Uint,
                (Format::R8Uint, 3) => Format::Rgb8Uint,
                (Format::R8Uint, 4) => Format::Rgba8Uint,
                (Format::R16Uint, 2) => Format::Rg16Uint,
                (Format::R16Uint, 3) => Format::Rgb16Uint,
                (Format::R16Uint, 4) => Format::Rgba16Uint,
                (Format::R32Uint, 2) => Format::Rg32Uint,
                (Format::R32Uint, 3) => Format::Rgb32Uint,
                (Format::R32Uint, 4) => Format::Rgba32Uint,
                (Format::R64Uint, 2) => Format::Rg64Uint,
                (Format::R64Uint, 3) => Format::Rgb64Uint,
                (Format::R64Uint, 4) => Format::Rgba64Uint,
                (Format::R32Float, 2) => Format::Rg32Float,
                (Format::R32Float, 3) => Format::Rgb32Float,
                (Format::R32Float, 4) => Format::Rgba32Float,
                (Format::R64Float, 2) => Format::Rg64Float,
                (Format::R64Float, 3) => Format::Rgb64Float,
                (Format::R64Float, 4) => Format::Rgba64Float,
                _ => {
                    failure::bail!(
                        "Unrecognized numeric array with component count {}",
                        traits.numeric.vector.component_count
                    );
                }
            },
        )
    } else {
        Ok(current_type)
    }
}

impl ReflectInto<gfx_hal::pso::Element<Format>> for ReflectTypeDescription {
    fn reflect_into(&self) -> Result<gfx_hal::pso::Element<Format>, failure::Error> {
        let format = type_element_format(self.type_flags, &self.traits)?;
        Ok(gfx_hal::pso::Element {
            format: format,
            offset: 0,
        })
    }
}

impl ReflectInto<gfx_hal::pso::AttributeDesc> for ReflectInterfaceVariable {
    fn reflect_into(&self) -> Result<gfx_hal::pso::AttributeDesc, failure::Error> {
        // An attribute is not an image format
        Ok(gfx_hal::pso::AttributeDesc {
            location: self.location,
            binding: self.location,
            element: self
                .type_description
                .as_ref()
                .ok_or_else(|| failure::format_err!("Unable to reflect vertex element"))?
                .reflect_into()?,
        })
    }
}

// Descriptor Sets
//

impl ReflectInto<gfx_hal::pso::DescriptorType> for ReflectDescriptorType {
    fn reflect_into(&self) -> Result<gfx_hal::pso::DescriptorType, failure::Error> {
        use gfx_hal::pso::DescriptorType;
        use ReflectDescriptorType::*;

        match *self {
            Sampler => Ok(DescriptorType::Sampler),
            CombinedImageSampler => Ok(DescriptorType::CombinedImageSampler),
            SampledImage => Ok(DescriptorType::SampledImage),
            StorageImage => Ok(DescriptorType::StorageImage),
            UniformTexelBuffer => Ok(DescriptorType::UniformTexelBuffer),
            StorageTexelBuffer => Ok(DescriptorType::StorageTexelBuffer),
            UniformBuffer => Ok(DescriptorType::UniformBuffer),
            StorageBuffer => Ok(DescriptorType::StorageBuffer),
            UniformBufferDynamic => Ok(DescriptorType::UniformBufferDynamic),
            StorageBufferDynamic => Ok(DescriptorType::StorageBufferDynamic),
            InputAttachment => Ok(DescriptorType::InputAttachment),
            AccelerationStructureNV => Err(failure::format_err!(
                "We cant handle AccelerationStructureNV descriptor type"
            )),
            Undefined => Err(failure::format_err!(
                "We cant handle undefined descriptor types"
            )),
        }
    }
}

impl ReflectInto<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>> for ReflectDescriptorSet {
    fn reflect_into(
        &self,
    ) -> Result<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>, failure::Error> {
        self.bindings
            .iter()
            .map(|desc| desc.reflect_into())
            .collect::<Result<Vec<_>, _>>()
    }
}
impl ReflectInto<gfx_hal::pso::DescriptorSetLayoutBinding> for ReflectDescriptorBinding {
    fn reflect_into(&self) -> Result<gfx_hal::pso::DescriptorSetLayoutBinding, failure::Error> {
        Ok(gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: self.binding,
            ty: self.descriptor_type.reflect_into()?,
            count: self.count as usize,
            stage_flags: gfx_hal::pso::ShaderStageFlags::VERTEX,
            immutable_samplers: false, // TODO: how to determine this?
        })
    }
}

fn convert_push_constant(
    stage: gfx_hal::pso::ShaderStageFlags,
    variable: &ReflectBlockVariable,
) -> Result<(gfx_hal::pso::ShaderStageFlags, std::ops::Range<u32>), failure::Error> {
    Ok((
        stage,
        variable.offset..variable.offset / 4 + variable.size / 4,
    ))
}

fn convert_stage(stage: ReflectShaderStageFlags) -> gfx_hal::pso::ShaderStageFlags {
    let mut bits = gfx_hal::pso::ShaderStageFlags::empty();

    if stage.contains(ReflectShaderStageFlags::VERTEX) {
        bits |= gfx_hal::pso::ShaderStageFlags::VERTEX;
    }
    if stage.contains(ReflectShaderStageFlags::FRAGMENT) {
        bits |= gfx_hal::pso::ShaderStageFlags::FRAGMENT;
    }
    if stage.contains(ReflectShaderStageFlags::GEOMETRY) {
        bits |= gfx_hal::pso::ShaderStageFlags::GEOMETRY;
    }
    if stage.contains(ReflectShaderStageFlags::COMPUTE) {
        bits |= gfx_hal::pso::ShaderStageFlags::COMPUTE;
    }
    if stage.contains(ReflectShaderStageFlags::TESSELLATION_CONTROL) {
        bits |= gfx_hal::pso::ShaderStageFlags::HULL;
    }
    if stage.contains(ReflectShaderStageFlags::TESSELLATION_EVALUATION) {
        bits |= gfx_hal::pso::ShaderStageFlags::DOMAIN;
    }

    bits
}

/// Implementation of shader reflection for SPIRV
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpirvShaderDescription {
    /// Hashmap of output variables with names.
    pub output_attributes: Vec<gfx_hal::pso::AttributeDesc>,
    /// Hashmap of output variables with names.
    pub input_attributes: Vec<gfx_hal::pso::AttributeDesc>,
    /// Hashmap of output variables with names.
    pub descriptor_sets: Vec<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>>,
    /// Stage flag of this shader
    pub stage_flag: gfx_hal::pso::ShaderStageFlags,
    /// Push Constants
    pub push_constants: Vec<(gfx_hal::pso::ShaderStageFlags, std::ops::Range<u32>)>,
    /// Raw shader bytes
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub spirv: Vec<u8>,
}

pub(crate) fn generate_attributes(
    attributes: Vec<ReflectInterfaceVariable>,
) -> Result<Vec<gfx_hal::pso::AttributeDesc>, failure::Error> {
    let mut out_attributes = Vec::new();

    for attribute in &attributes {
        let reflected: gfx_hal::pso::AttributeDesc = attribute.reflect_into()?;
        if attribute.array.dims.is_empty() {
            out_attributes.push(reflected);
        } else {
            for n in 0..attribute.array.dims[0] {
                let mut clone = reflected.clone();
                clone.location += n;
                out_attributes.push(clone);
            }
        }
    }

    Ok(out_attributes)
}

impl Shader for SpirvShaderDescription {
    fn spirv(&self) -> Result<std::borrow::Cow<'_, [u8]>, failure::Error> {
        Ok(std::borrow::Cow::Borrowed(&self.spirv))
    }

    fn reflect(&self) -> Result<&SpirvShaderDescription, failure::Error> {
        Ok(self)
    }
}

impl SpirvShaderDescription {
    /// Creates a reflection instance based on the provided spirv byte code
    pub fn from_bytes(data: &[u8]) -> Result<Self, failure::Error> {
        log::trace!("Shader reflecting into SpirvShaderDescription");

        match ShaderModule::load_u8_data(data) {
            Ok(module) => {
                let stage_flag = convert_stage(module.get_shader_stage());

                let input_attributes =
                    generate_attributes(module.enumerate_input_variables(None).map_err(|e| {
                        failure::format_err!(
                            "Failed to get input attributes from spirv-reflect: {}",
                            e
                        )
                    })?);

                let output_attributes =
                    generate_attributes(module.enumerate_input_variables(None).map_err(|e| {
                        failure::format_err!(
                            "Failed to get output attributes from spirv-reflect: {}",
                            e
                        )
                    })?);

                let descriptor_sets: Result<Vec<_>, _> = module
                    .enumerate_descriptor_sets(None)
                    .map_err(|e| {
                        failure::format_err!(
                            "Failed to get descriptor sets from spirv-reflect: {}",
                            e
                        )
                    })?
                    .iter()
                    .map(ReflectInto::<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>>::reflect_into)
                    .collect();

                // This is a fixup-step required because of our implementation. Because we dont pass the module around
                // to the all the reflect_into API's, we need to fix up the shader stage here at the end. Kinda a hack
                let mut descriptor_sets_final = descriptor_sets
                    .map_err(|e| failure::format_err!("Failed to parse descriptor sets:: {}", e))?;
                descriptor_sets_final.iter_mut().for_each(|v| {
                    v.iter_mut()
                        .for_each(|mut set| set.stage_flags = stage_flag);
                });

                let push_constants: Result<Vec<_>, _> = module
                    .enumerate_push_constant_blocks(None)
                    .map_err(|e| {
                        failure::format_err!(
                            "Failed to get push constants from spirv-reflect: {}",
                            e
                        )
                    })?
                    .iter()
                    .map(|c| convert_push_constant(stage_flag, c))
                    .collect();

                Ok(Self {
                    input_attributes: input_attributes.map_err(|e| {
                        failure::format_err!("Error parsing input attributes: {}", e)
                    })?,
                    output_attributes: output_attributes.map_err(|e| {
                        failure::format_err!("Error parsing output attributes: {}", e)
                    })?,
                    descriptor_sets: descriptor_sets_final,
                    push_constants: push_constants
                        .map_err(|e| failure::format_err!("Error parsing push constants: {}", e))?,
                    stage_flag,
                    spirv: data.to_vec(),
                })
            }
            Err(e) => Err(failure::format_err!("Failed to reflect data: {}", e)),
        }
    }
}
