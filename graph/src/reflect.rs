/// Reflection extensions

use rendy_shader::reflect::SpirvShaderDescription;
use crate::node::render::{Layout, SetLayout};

/// Extension for SpirvShaderReflection providing graph render type conversion
pub trait ShaderLayoutGenerator {
    /// Convert reflected descriptor sets to a Layout structure
    fn layout(&self) -> Result<Layout, failure::Error>;

    /// Convert reflected attributes to a direct gfx_hal element array
    fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride);
}

impl ShaderLayoutGenerator for SpirvShaderDescription {
    fn layout(&self) -> Result<Layout, failure::Error> {
        Ok(Layout {
            sets: self.descriptor_sets.iter().map(|set| SetLayout { bindings: set.clone() }).collect(),
            push_constants: Vec::new(),
        })
    }

    fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride)
    {
        let stride: u32 = 0;
        let elements: Vec<gfx_hal::pso::Element<gfx_hal::format::Format>> = self.input_attributes.iter()
            .map(|v| { v.element } )
            .collect();

        (elements, stride)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BindingEquality {
    Equal,
    SameBindingNonEqual,
    NotEqual,
}

pub fn compare_bindings(lhv: &gfx_hal::pso::DescriptorSetLayoutBinding, rhv: &gfx_hal::pso::DescriptorSetLayoutBinding) -> BindingEquality {
    if lhv.binding == rhv.binding
        && lhv.count == rhv.count
        && lhv.immutable_samplers == rhv.immutable_samplers
        && lhv.ty == rhv.ty {
        return BindingEquality::Equal;
    } else {
        if lhv.binding == rhv.binding {
            return BindingEquality::SameBindingNonEqual;
        }
    }

    return BindingEquality::NotEqual;
}

impl ShaderLayoutGenerator for (SpirvShaderDescription, SpirvShaderDescription) {
    fn layout(&self) -> Result<Layout, failure::Error> {
        let mut set_layouts = Vec::new();

        for (n, set_1) in self.0.descriptor_sets.iter().enumerate() {
            let mut out_set = Vec::new();
            for (_, set_2) in self.1.descriptor_sets.iter().enumerate() {
                if n <= set_2.len() { // We have matching sets, do they have matching bindings?
                    for descriptor_1 in set_1 {
                        for descriptor_2 in set_2 {
                            match compare_bindings(descriptor_1, descriptor_2) {
                                BindingEquality::Equal => {
                                    // Change the binding type to graphics because its both stages
                                    let mut copy = descriptor_1.clone();
                                    copy.stage_flags = gfx_hal::pso::ShaderStageFlags::GRAPHICS;
                                    out_set.push(copy);
                                },
                                BindingEquality::SameBindingNonEqual => {
                                    return Err(failure::format_err!( "Descriptor binding @ (binding: {}, index={}) mismatch between the two shaders. This usually means there is a binding conflict between the two shaders.",
                                    descriptor_1.binding,
                                    n));
                                },
                                BindingEquality::NotEqual => {
                                    out_set.push(descriptor_1.clone());
                                },
                            };
                        }
                    }
                }
            }

            self.1.descriptor_sets.iter().for_each(|set| {
                set.iter().for_each(|descriptor| {
                    if let None = out_set.iter().find(|v| compare_bindings(v, descriptor) == BindingEquality::Equal) {
                        out_set.push(descriptor.clone());
                    }
                });
            });

            set_layouts.push(SetLayout { bindings: out_set } );
        }

        Ok(Layout {
            sets: set_layouts,
            push_constants: Vec::new(),
        })
    }

    fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride) {
        let stride: u32 = 0;
        let elements: Vec<gfx_hal::pso::Element<gfx_hal::format::Format>> = self.0.input_attributes.iter()
            .map(|v| { v.element } ).collect();

        (elements, stride)
    }
}