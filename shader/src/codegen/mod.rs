//! asdf

use crate::ShaderSetBuilder;

use quote::quote;

use rustfmt_nightly::{Config, Edition, EmitMode, Input, Session};

pub trait PodGenerator {
    fn generate_pods(&self,) -> String;

}

impl PodGenerator for ShaderSetBuilder {
    fn generate_pods(&self, ) -> String {

        // Get the descriptor sets and generate pods for them

        

        let pod = quote! {
            use rendy_util::types::vertex::{Attribute, AsAttribute};
            use std::{borrow::Cow, fmt::Debug};

            fn hello_world() {
                println!("Hello World!");
            }
        };

        // Format the code
        let output = rustfmt(pod.to_string());



        panic!("out={}", output.unwrap());

    }
}

/// Programmatically runs rustfmt on a `String`.
pub fn rustfmt<S>(module: S) -> Result<String, failure::Error>
    where
        S: Into<String>,
{
    let input = Input::Text(module.into());

    let mut config = Config::default();
    config.set().edition(Edition::Edition2018);
    config.set().emit_mode(EmitMode::Stdout);

    let mut output = Vec::new();
    {
        let mut session = Session::new(config, Some(&mut output));
        let _format_report = session.format(input)?;
    }
    let s = String::from_utf8(output)?;
    Ok(s)
}