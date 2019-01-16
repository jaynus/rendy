//! Framegraph implementation for Rendy engine.

#![forbid(overflowing_literals)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(intra_doc_link_resolution_failure)]
#![deny(path_statements)]
#![deny(trivial_bounds)]
#![deny(type_alias_bounds)]
#![deny(unconditional_recursion)]
#![deny(unions_with_drop_fields)]
#![deny(while_true)]
#![deny(unused)]
#![deny(bad_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![allow(unused_unsafe)]

use rendy_chain as chain;
use rendy_command as command;
use rendy_factory as factory;
use rendy_frame as frame;
use rendy_memory as memory;
use rendy_resource as resource;
use rendy_wsi as wsi;

/// Id of the buffer in graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferId(usize);

/// Id of the image (or target) in graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageId(usize);

/// Id of the node in graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(usize);

mod graph;
mod node;

pub use self::{graph::*, node::*};

/// Reflection extensions
pub mod reflect {
    #[cfg(feature = "reflection")]
    use rendy_shader::reflect::SpirvShaderDescription;

    #[cfg(feature = "reflection")]
    use crate::node::render::{Layout, SetLayout};

    /// Extension for SpirvShaderReflection providing graph render type conversion
    #[cfg(feature = "reflection")]
    pub trait ShaderReflectBuilder {
        /// Convert reflected descriptor sets to a Layout structure
        fn layout(&self) -> Layout;

        /// Convert reflected attributes to a direct gfx_hal element array
        fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride);
    }

    #[cfg(feature = "reflection")]
    impl ShaderReflectBuilder for SpirvShaderDescription {
        fn layout(&self) -> Layout {
            use rendy_shader::reflect::AsVector;

            let sets = self.descriptor_sets.iter().map(|set| { SetLayout { bindings: set.as_vector() }  }).collect();

            Layout {
                sets,
                push_constants: Vec::new(),
            }
        }

        fn attributes(&self) -> (Vec<gfx_hal::pso::Element<gfx_hal::format::Format>>, gfx_hal::pso::ElemStride)
        {
            let stride: u32 = 0;
            let elements: Vec<gfx_hal::pso::Element<gfx_hal::format::Format>> = self.input_variables.iter()
                .filter(|(k, _)|{
                    if k.contains("gl_") || k.is_empty() {
                        return false
                    }
                    true
                })
                .map(|(_, v)| {
                    v.element
                } ).collect();

            (elements, stride)
        }
    }
}