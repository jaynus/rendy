//! Using spirv-reflect-rs for reflection.
//!

use std::fs::File;
use std::io::prelude::*;

use spirv_reflect::{
    ShaderModule,
    types::*,
};

use gfx_hal::format::Format;

/// Workaround extension trait copy of std::convert::From, for simple conversion from spirv-reflect types to gfx_hal types
trait ReflectInto<T>: Sized {
    /// Attempts to perform a conversion from the provided type into this type
    fn reflect_into(&self) -> Result<T, ReflectError> {
        Err("Unsupported conversion type".into())
    }
}

impl ReflectInto<Format> for image::ReflectFormat {
    fn reflect_into(&self, ) -> Result<Format, ReflectError> {
        match self {
            image::ReflectFormat::Undefined => Err("Undefined format".into()),
            image::ReflectFormat::R32_UINT => Ok(Format::R32Uint),
            image::ReflectFormat::R32_SINT => Ok(Format::R32Int),
            image::ReflectFormat::R32_SFLOAT => Ok(Format::R32Float),
            image::ReflectFormat::R32G32_UINT => Ok(Format::Rg32Uint),
            image::ReflectFormat::R32G32_SINT => Ok(Format::Rg32Int),
            image::ReflectFormat::R32G32_SFLOAT => Ok(Format::Rg32Float),
            image::ReflectFormat::R32G32B32_UINT => Ok(Format::Rgb32Uint),
            image::ReflectFormat::R32G32B32_SINT => Ok(Format::Rgb32Int),
            image::ReflectFormat::R32G32B32_SFLOAT => Ok(Format::Rgb32Float),
            image::ReflectFormat::R32G32B32A32_UINT => Ok(Format::Rgb32Uint),
            image::ReflectFormat::R32G32B32A32_SINT => Ok(Format::Rgb32Int),
            image::ReflectFormat::R32G32B32A32_SFLOAT => Ok(Format::Rgb32Float),
        }
    }
}

impl ReflectInto<gfx_hal::pso::Element<gfx_hal::format::Format>> for variable::ReflectTypeDescription {
    fn reflect_into(&self, ) -> Result<gfx_hal::pso::Element<gfx_hal::format::Format>, ReflectError> {
        match self.type_flags {
            _ => {
                let element = gfx_hal::pso::Element { format: Format::Rgb32Uint, offset: 0, };
                Ok(element)
            },
        }
    }
}

impl ReflectInto<gfx_hal::pso::AttributeDesc> for variable::ReflectInterfaceVariable {
    fn reflect_into(&self, ) -> Result<gfx_hal::pso::AttributeDesc, ReflectError> {
        // An attribute is not an image format
        println!("Desc: {:?}", self.type_description);
        Ok(gfx_hal::pso::AttributeDesc {
            location: self.location,
            binding: self.location,
            //element: gfx_hal::pso::Element { format: self.format.reflect_into()?, offset: 0, },
            element: gfx_hal::pso::Element { format: Format::Rgb32Uint, offset: 0 }, // TODO: elements
        })
    }
}

// Descriptor Sets
//


impl ReflectInto<gfx_hal::pso::DescriptorType> for descriptor::ReflectDescriptorType {
    fn reflect_into(&self, ) -> Result<gfx_hal::pso::DescriptorType, ReflectError> {
        match *self {
            descriptor::ReflectDescriptorType::Sampler => Ok(gfx_hal::pso::DescriptorType::Sampler),
            descriptor::ReflectDescriptorType::CombinedImageSampler => Ok(gfx_hal::pso::DescriptorType::CombinedImageSampler),
            descriptor::ReflectDescriptorType::SampledImage => Ok(gfx_hal::pso::DescriptorType::SampledImage),
            descriptor::ReflectDescriptorType::StorageImage => Ok(gfx_hal::pso::DescriptorType::StorageImage),
            descriptor::ReflectDescriptorType::UniformTexelBuffer => Ok(gfx_hal::pso::DescriptorType::UniformTexelBuffer),
            descriptor::ReflectDescriptorType::StorageTexelBuffer => Ok(gfx_hal::pso::DescriptorType::StorageTexelBuffer),
            descriptor::ReflectDescriptorType::UniformBuffer => Ok(gfx_hal::pso::DescriptorType::UniformBuffer),
            descriptor::ReflectDescriptorType::StorageBuffer => Ok(gfx_hal::pso::DescriptorType::StorageBuffer),
            descriptor::ReflectDescriptorType::UniformBufferDynamic => Ok(gfx_hal::pso::DescriptorType::UniformBufferDynamic),
            descriptor::ReflectDescriptorType::StorageBufferDynamic => Ok(gfx_hal::pso::DescriptorType::StorageBufferDynamic),
            descriptor::ReflectDescriptorType::InputAttachment => Ok(gfx_hal::pso::DescriptorType::InputAttachment),
            descriptor::ReflectDescriptorType::Undefined => Err("We cant handle undefined descriptor types".into()),
        }
    }
}

impl ReflectInto<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>> for descriptor::ReflectDescriptorSet {
    fn reflect_into(&self, ) -> Result<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>, ReflectError> {
        let mut output = Vec::<gfx_hal::pso::DescriptorSetLayoutBinding>::new();

        for descriptor in self.bindings.iter() {
            output.push(descriptor.reflect_into()?);
        }

        Ok(output)
    }
}
impl ReflectInto<gfx_hal::pso::DescriptorSetLayoutBinding> for descriptor::ReflectDescriptorBinding {
    fn reflect_into(&self, ) -> Result<gfx_hal::pso::DescriptorSetLayoutBinding, ReflectError> {
        Ok(gfx_hal::pso::DescriptorSetLayoutBinding {
            binding: self.binding,
            ty: self.descriptor_type.reflect_into()?,
            count: self.count as usize,
            stage_flags: gfx_hal::pso::ShaderStageFlags::VERTEX,
            immutable_samplers: false, // TODO: how to determine this?
        })
    }
}

#[derive(Debug, failure::Fail)]
/// Reflection error types
pub enum ReflectError {
    /// spirv-reflect errors
    #[fail(display = "{}", _0)]
    ErrorMessage(String),

    /// IO Errors when reading shaders
    #[fail(display = "{}", _0)]
    IoError(std::io::Error),
}
impl From<&str> for ReflectError {
    fn from(error: &str) -> Self {
        ReflectError::ErrorMessage(error.to_string())
    }
}
impl From<std::io::Error> for ReflectError {
    fn from(error: std::io::Error) -> Self {
        ReflectError::IoError(error)
    }
}


/// trait for accessing reflection of a given shader.
pub trait ShaderDescription {
    /// Returns slice of shader input variables
    fn inputs(&self, ) -> &[gfx_hal::pso::AttributeDesc];

    /// Returns slice of shader output variables
    fn outputs(&self, ) -> &[gfx_hal::pso::AttributeDesc];

    /// Returns a slice of the descriptor set vectors
    fn descriptor_sets(&self, ) -> &[Vec<gfx_hal::pso::DescriptorSetLayoutBinding>];
}

/// Implementation of shader reflection for SPIRV
#[derive(Clone, Debug)]
pub struct SpirvShaderDescription {
    output_variables: Vec<gfx_hal::pso::AttributeDesc>,
    input_variables: Vec<gfx_hal::pso::AttributeDesc>,
    descriptor_sets: Vec<Vec<gfx_hal::pso::DescriptorSetLayoutBinding>>,
}


impl SpirvShaderDescription {
    ///
    pub fn from_bytes(data: &[u8]) -> Result<Self, ReflectError> {
        match ShaderModule::load_u8_data(data) {
            Ok(module) => {

                //let desc = module.enumerate_descriptor_sets(None).unwrap();
                //let desc_b = module.enumerate_descriptor_bindings(None).unwrap();
                //println!("{:?}\n{:?}",desc, desc_b);
                Ok(Self{
                    // TODO: change these unwraps back to actual error checking
                    input_variables: module.enumerate_input_variables(None)?.iter().map(|v| v.reflect_into().unwrap() ).collect(),
                    output_variables: module.enumerate_output_variables(None)?.iter().map(|v| v.reflect_into().unwrap() ).collect(),
                    descriptor_sets: module.enumerate_descriptor_sets(None)?.iter().map(|v| v.reflect_into().unwrap() ).collect(),
                })
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    ///
    pub fn from_file<P>(path: P) -> Result<Self, ReflectError>
        where
            P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut file = File::open(path)?;
        let mut contents: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut contents)?;

        Self::from_bytes(contents.as_slice())
    }
}

impl  ShaderDescription for SpirvShaderDescription
{
    fn descriptor_sets(&self, ) -> &[Vec<gfx_hal::pso::DescriptorSetLayoutBinding>]  { //Vec<>
        self.descriptor_sets.as_slice()
    }

    fn inputs(&self, ) -> &[gfx_hal::pso::AttributeDesc] {
        self.input_variables.as_slice()
    }

    fn outputs(&self, ) -> &[gfx_hal::pso::AttributeDesc] {
        self.output_variables.as_slice()
    }
}