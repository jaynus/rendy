/// Reflection extensions

use rendy_shader::reflect::SpirvShaderDescription;
use crate::node::render::{Layout, SetLayout};

/// Extension for SpirvShaderReflection providing graph render type conversion
/// Implementors of this return the appropriate descriptor sets and attribute layers for a given shader set.
// this lives in graph instead of Shader due to not wanting to pull in all the layout requirements and cause a cross-dependency with rendy-shader
pub trait ShaderLayoutGenerator {
    /// Convert reflected descriptor sets to a Layout structure
    fn layout(&self) -> Result<Layout, failure::Error>;

    /// Convert reflected attributes to a direct gfx_hal element array
    fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride);
}

///
/// This implementation lives to reflect a single shader description into a usable gfx layout
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



///
/// This implementation lives to merge two shader reflections into a single layout and attribute descriptor.
/// This will be the most commonly used implementation of [ShaderLayoutGenerator], as it is capable of merging and mapping
/// descriptors for a Vertex+Fragment shader pair.
impl<S> ShaderLayoutGenerator for (S, S)
    where S: ShaderLayoutGenerator + Sized
{
    fn layout(&self) -> Result<Layout, failure::Error> {
        let mut set_layouts = Vec::new();

        let first_layout = self.0.layout()?;
        let second_layout = self.1.layout()?;

        log::trace!("first sets: {}", self.0.layout()?.sets.len());

        for (n, set_1) in first_layout.sets.iter().enumerate() {
            let mut out_set = Vec::new();
            log::trace!("second sets: {}", self.1.layout()?.sets.len());

            if ! second_layout.sets.is_empty() {
                for (_, set_2) in second_layout.sets.iter().enumerate() {
                    if n <= set_2.bindings.len() { // We have matching sets, do they have matching bindings?
                        log::trace!("Set had bindings");
                        for descriptor_1 in &set_1.bindings {
                            for descriptor_2 in &set_2.bindings {
                                log::trace!("iter");
                                match compare_bindings(descriptor_1, descriptor_2) {
                                    BindingEquality::Equal => {
                                        // Change the binding type to graphics because its both stages
                                        let mut copy = descriptor_1.clone();
                                        copy.stage_flags = gfx_hal::pso::ShaderStageFlags::GRAPHICS;
                                        out_set.push(copy);
                                    },
                                    BindingEquality::SameBindingNonEqual => {
                                        // We throw an error here because it means we found a binding
                                        // in both shaders that has the same binding number, but different descriptions.
                                        // Therefore its user error.
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
            } else {
                self.0.layout()?.sets.iter().for_each(|set| {
                    set.bindings.iter().for_each(|descriptor| { out_set.push(descriptor.clone()); });
                });
            }

            set_layouts.push(SetLayout { bindings: out_set } );
        }

        // After iterating the first shaders binding set (vertex), we THEN iterate the second shader (fragment usually)
        // And only add descriptor sets which were not already added in the vertex shader. We do this because they can
        // share descriptor sets or partials
        let mut out_set = Vec::new();
        self.1.layout()?.sets.iter().for_each(|set| {
            set.bindings.iter().for_each(|descriptor| {
                if let None = out_set.iter().find(|v| compare_bindings(v, descriptor) == BindingEquality::Equal) {
                    out_set.push(descriptor.clone());
                }
            });
        });

        log::trace!("Reflecting: {:?}", out_set);
        if out_set.len() > 0 {
            set_layouts.push(SetLayout { bindings: out_set } );
        }

        Ok(Layout {
            sets: set_layouts,
            push_constants: Vec::new(),
        })
    }

    fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride) {
        self.0.attributes()
    }
}

pub fn merge_descriptor_sets<'a, I>(mut layouts: I) -> Result<Layout, failure::Error>
    where I: Iterator<Item = &'a dyn ShaderLayoutGenerator>,
{
    layouts.next().unwrap().layout()
}


/// This enum provides logical comparison results for descriptor sets. Because shaders can share bindings,
/// we cannot do a strict equality check for exclusion - we must see if shaders match, or if they are the same bindings
/// but mismatched descriptions.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BindingEquality {
    /// The bindings share a binding index, but have different values. This is usually an error case.
    Equal,
    /// The bindings match
    SameBindingNonEqual,
    /// The bindings do not equal
    NotEqual,
}

/// Logically compares two descriptor layout bindings to determine their relational equality.
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
