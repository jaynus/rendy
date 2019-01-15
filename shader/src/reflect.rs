//! Using spirv-reflect-rs for reflection.
//!

use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use spirv_reflect::{
    ShaderModule,
    types::*,
};

use gfx_hal::format::Format;

/// Workaround extension trait copy of std::convert::From, for simple conversion from spirv-reflect types to gfx_hal types
pub trait ReflectInto<T>: Sized {
    /// Attempts to perform a conversion from the provided type into this type
    fn reflect_into(&self) -> Result<T, ReflectError> {
        Err("Unsupported conversion type".into())
    }
}

/// Harness type for easier conversions of named return collections.
pub trait AsVector<V> {
    /// Implemented to return a straight vector from a hashmap, so the user doesnt have to map.collect for all its uses
    /// This function clones all values in the hashmap so beware.
    fn as_vector(&self, ) -> Vec<V>;
}

impl<K, V> AsVector<V> for HashMap<K, V>
    where
        K: Eq + std::hash::Hash,
        V: Sized + Clone,
{
    fn as_vector(&self, ) -> Vec<V> {
        self.into_iter().map(|(_, i)| { (*i).clone() }).collect()
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

    /// Option errors
    #[fail(display = "NoneError")]
    NoneError(std::option::NoneError),
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
impl From<std::option::NoneError> for ReflectError {
    fn from(error: std::option::NoneError) -> Self {
        ReflectError::NoneError(error)
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

fn type_element_format(flags: variable::ReflectTypeFlags, traits: &traits::ReflectTypeDescriptionTraits) -> gfx_hal::format::Format {
    let mut current_type = Format::R32Float;

    if flags.contains(variable::ReflectTypeFlags::INT) {
        // TODO: rendy/gfx doesnt seem to support non-32 bit formats?
        // TODO: support other bits
        assert_eq!(traits.numeric.scalar.width, 32);

        current_type = match traits.numeric.scalar.signedness {
            1 => match traits.numeric.scalar.width {
                8 => Format::R8Int,
                16 => Format::R16Int,
                32 => Format::R32Int,
                64 => Format::R64Int,
                _ => panic!("Unrecognized scalar width for int"),
            },
            0 => match traits.numeric.scalar.width {
                8 => Format::R8Uint,
                16 => Format::R16Uint,
                32 => Format::R32Uint,
                64 => Format::R64Uint,
                _ => panic!("Unrecognized scalar width for unsigned int"),
            },
            _ => panic!("LOL"),
        };
    }
    if flags.contains(variable::ReflectTypeFlags::FLOAT) {
        // TODO: support other bits
        current_type = match traits.numeric.scalar.width {
            32 => Format::R32Float,
            64 => Format::R64Float,
            _ => panic!("Unrecognized scalar width for float"),
        }
    }

    if flags.contains(variable::ReflectTypeFlags::VECTOR) {
        current_type = match traits.numeric.vector.component_count {
            2 => match current_type {
                Format::R64Float => Format::Rg64Float,
                Format::R32Float => Format::Rg32Float,
                Format::R32Int => Format::Rg32Int,
                Format::R32Uint => Format::Rg32Int,
                _ => panic!("LOL: {:?}", current_type),
            },
            3 => match current_type {
                Format::R64Float => Format::Rgb64Float,
                Format::R32Float => Format::Rgb32Float,
                Format::R32Int => Format::Rgb32Int,
                Format::R32Uint => Format::Rgb32Int,
                _ => panic!("LOL: {:?}", current_type),
            },
            4 => match current_type {
                Format::R64Float => Format::Rgba64Float,
                Format::R32Float => Format::Rgba32Float,
                Format::R32Int => Format::Rgba32Int,
                Format::R32Uint => Format::Rgba32Int,
                _ => panic!("LOL: {:?}", current_type),
            },
            _ => panic!("LOL: {:?}", traits.numeric.vector.component_count),
        };
    }

    current_type
}

impl ReflectInto<gfx_hal::pso::Element<gfx_hal::format::Format>> for variable::ReflectTypeDescription {
    fn reflect_into(&self, ) -> Result<gfx_hal::pso::Element<gfx_hal::format::Format>, ReflectError> {
        Ok(gfx_hal::pso::Element { format: type_element_format(self.type_flags, &self.traits), offset: 0, })
    }
}

impl ReflectInto<(String, gfx_hal::pso::AttributeDesc)> for variable::ReflectInterfaceVariable {
    fn reflect_into(&self) -> Result<(String, gfx_hal::pso::AttributeDesc), ReflectError> {
        // An attribute is not an image format
        Ok((self.name.clone(), gfx_hal::pso::AttributeDesc {
            location: self.location,
            binding: self.location,
            //element: gfx_hal::pso::Element { format: self.format.reflect_into()?, offset: 0, },
            element: self.type_description.as_ref()?.reflect_into()?, // TODO: elements
        }))
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

impl ReflectInto<HashMap<String, gfx_hal::pso::DescriptorSetLayoutBinding>> for descriptor::ReflectDescriptorSet {
    fn reflect_into(&self, ) -> Result<HashMap<String, gfx_hal::pso::DescriptorSetLayoutBinding>, ReflectError> {
        let mut output = HashMap::<String, gfx_hal::pso::DescriptorSetLayoutBinding>::new();

        for descriptor in self.bindings.iter() {
            assert!(!output.contains_key(&descriptor.name));
            output.insert(descriptor.name.clone(), descriptor.reflect_into()?);
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



/// Implementation of shader reflection for SPIRV
#[derive(Clone)]
pub struct SpirvShaderDescription {
    /// Hashmap of output variables with names.
    pub output_variables: HashMap<String, gfx_hal::pso::AttributeDesc>,
    /// Hashmap of output variables with names.
    pub input_variables: HashMap<String, gfx_hal::pso::AttributeDesc>,
    /// Hashmap of output variables with names.
    pub descriptor_sets: Vec<HashMap<String, gfx_hal::pso::DescriptorSetLayoutBinding>>,
}


impl SpirvShaderDescription {
    ///
    pub fn from_bytes(data: &[u8], strict: bool) -> Result<Self, ReflectError> {
        match ShaderModule::load_u8_data(data) {
            Ok(module) => {
                Ok(Self{
                    // TODO: change these unwraps back to actual error checking
                    // TODO: strict isnt really strict
                    input_variables: module.enumerate_input_variables(None)?.iter()
                        .filter(|v| {
                            let r = v.reflect_into();
                            match strict {
                                true => { r.is_ok() && !r.as_ref().unwrap().0.is_empty() },
                                false => r.is_ok(),
                            }
                        })
                        .map(|v| {
                            v.reflect_into().unwrap()
                        })
                        .collect(),
                    output_variables: module.enumerate_output_variables(None)?.iter()
                        .filter(|v| {
                            let r = v.reflect_into();
                            match strict {
                                true => { r.is_ok() && !r.as_ref().unwrap().0.is_empty() },
                                false => r.is_ok(),
                            }
                        })
                        .map(|v| {
                            v.reflect_into().unwrap()
                        }).
                        collect(),
                    descriptor_sets: module.enumerate_descriptor_sets(None)?.iter()
                        .filter(|v| {
                            match strict {
                                true => { v.reflect_into().unwrap(); true },
                                false => v.reflect_into().is_ok(),
                            }
                        })
                        .map(|v| {
                            v.reflect_into().unwrap()
                        })
                        .collect(),
                })
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    ///
    pub fn from_file<P>(path: P, strict: bool) -> Result<Self, ReflectError>
        where
            P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut file = File::open(path)?;
        let mut contents: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut contents)?;

        Self::from_bytes(contents.as_slice(), strict)
    }
}

impl std::fmt::Debug for SpirvShaderDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for input in &self.input_variables {
            write!(f, "input: {:?}\n", input)?;
        }

        for output in &self.output_variables {
            write!(f, "output: {:?}\n", output)?;
        }

        for output in &self.descriptor_sets {
            write!(f, "descriptors: {:?}\n", output)?;
        }
        Ok(())
    }
}