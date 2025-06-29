// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::ShaderReflection;

#[derive(Debug, Default)]
pub struct Pipeline {
    pub name: String,
    pub vert_path: PathBuf,
    pub frag_path: PathBuf,
    pub shaders: Vec<Shader>,
}

impl Pipeline {
    pub fn new<S: Into<String>>(name: S, reflections: &[ShaderReflection]) -> Self {
        let mut shaders = Vec::new();
        assert_eq!(reflections.len(), 2);

        for reflection in reflections {
            let entry_point_count = reflection.get_entry_point_count();
            for i in 0..entry_point_count {
                if let Some(entry_point) = reflection.get_entry_point_by_index(i) {
                    let stage = entry_point.get_stage();
                    match stage {
                        slang::Stage::Vertex => shaders.push(Shader::new(ShaderType::Vertex)),
                        slang::Stage::Fragment => shaders.push(Shader::new(ShaderType::Fragment)),
                        _ => panic!("{}:{}: Unimplemented stage {:?}", file!(), line!(), stage),
                    }
                }
            }
        }

        Pipeline {
            name: name.into(),
            vert_path: reflections[0].path.clone(),
            frag_path: reflections[1].path.clone(),
            shaders,
        }
    }
}

#[derive(Debug, Default)]
pub struct Shader {
    pub ty: ShaderType,
}

impl Shader {
    pub fn new(ty: ShaderType) -> Self {
        Self { ty }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum ShaderType {
    #[default]
    Vertex,
    Fragment,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn model() {
        let reflections = [
            ShaderReflection::parse("shaders/simple.vert.slang"),
            ShaderReflection::parse("shaders/simple.frag.slang"),
        ];
        let pipeline = Pipeline::new("Test", &reflections);
        assert_eq!(pipeline.name, "Test");
        assert_eq!(pipeline.shaders.len(), 2);
        assert_eq!(pipeline.shaders[0].ty, ShaderType::Vertex);
        assert_eq!(pipeline.shaders[1].ty, ShaderType::Fragment);
    }
}
