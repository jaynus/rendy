//! Using spirv-reflect-rs for reflection.
//!

use std::fs::File;
use std::io::prelude::*;

use spirv_reflect::{
    ShaderModule,
    types::*,
};

#[derive(Debug, failure::Fail)]
/// Reflection error types
pub enum ReflectError {
    /// spirv-reflect errors
    #[fail(display = "{}", _0)]
    SpirvReflectError(String),

    /// IO Errors when reading shaders
    #[fail(display = "{}", _0)]
    IoError(std::io::Error),
}
impl From<&str> for ReflectError {
    fn from(error: &str) -> Self {
        ReflectError::SpirvReflectError(error.to_string())
    }
}
impl From<std::io::Error> for ReflectError {
    fn from(error: std::io::Error) -> Self {
        ReflectError::IoError(error)
    }
}


/// trait describing a given shader attribute (input/output variable)
pub trait Attribute {
    /// Debug name of the trait, if accessible
    fn name(&self) -> Option<&String>;

    /// The complete size of the attribute, in bytes
    fn size(&self) -> u16;

    /// The format of the attribute
    fn format(&self) -> gfx_hal::format::Format;

    /// The type of attribute
    fn storage_class(&self) -> variable::ReflectStorageClass;

    /// Attribute 'location' descriptor
    fn location(&self) -> u8;

    /// Decoration flags of the variable
    fn flags(&self) -> variable::ReflectDecorationFlags;
}

/// Attribute implementation for spirv-reflect
#[derive(Clone, PartialEq, Debug)]
pub struct SpirvAttribute {
    name: Option<String>,
    size: u16,
    format: gfx_hal::format::Format,
    storage_class: variable::ReflectStorageClass,
    location: u8,
    flags: variable::ReflectDecorationFlags,
}

impl SpirvAttribute {
    /// Create a new attribute description from internal spirv-reflect type
    pub fn new(variable: &variable::ReflectInterfaceVariable) -> Self {
        Self {
            name: match variable.name.is_empty() { true => None, false => Some(variable.name.to_string()), },
            size: 0, // calculate
            format: gfx_hal::format::Format::A1r5g5b5Unorm, // Convert and calculate
            storage_class: variable.storage_class,
            location: variable.location as u8,
            flags: variable.decoration_flags,
        }
    }
}

impl Attribute for SpirvAttribute {
    fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn size(&self) -> u16 {
        self.size
    }

    fn format(&self) -> gfx_hal::format::Format {
        self.format
    }

    fn storage_class(&self) -> variable::ReflectStorageClass {
        self.storage_class
    }

    fn location(&self) -> u8 {
        self.location
    }

    fn flags(&self) -> variable::ReflectDecorationFlags {
        self.flags
    }
}


/// trait for accessing reflection of a given shader.
pub trait ShaderDescription {
    /// Returns slice of shader input variables
    fn inputs(&self, ) -> &[SpirvAttribute];

    /// Returns slice of shader output variables
    fn outputs(&self, ) -> &[SpirvAttribute];
}

/// Implementation of shader reflection for SPIRV
#[derive(Clone, Debug)]
pub struct SpirvShaderDescription {
    output_variables: Vec<SpirvAttribute>,
    input_variables: Vec<SpirvAttribute>,
}


impl SpirvShaderDescription {
    ///
    pub fn from_bytes(data: &[u8]) -> Result<Self, ReflectError> {
        match ShaderModule::load_u8_data(data) {
            Ok(module) => {
                let input_variables = module.enumerate_input_variables(None)?;
                let output_variables = module.enumerate_output_variables(None)?;

                Ok(Self{
                    input_variables: input_variables.iter().map(|v| SpirvAttribute::new(v)).collect(),
                    output_variables: output_variables.iter().map(|v| SpirvAttribute::new(v)).collect(),
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
    fn inputs(&self, ) -> &[SpirvAttribute] {
        self.input_variables.as_slice()
    }

    fn outputs(&self, ) -> &[SpirvAttribute] {
        self.output_variables.as_slice()
    }
}