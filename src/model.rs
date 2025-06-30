// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::ShaderReflection;

#[derive(Default)]
pub struct PipelineBuilder<'a> {
    name: String,
    shaders: Vec<ShaderReflection<'a>>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    pub fn vert(mut self, vert: ShaderReflection<'a>) -> Self {
        self.shaders.push(vert);
        self
    }

    pub fn frag(mut self, frag: ShaderReflection<'a>) -> Self {
        self.shaders.push(frag);
        self
    }

    pub fn build(self) -> Pipeline {
        assert!(!self.name.is_empty());
        assert!(!self.shaders.is_empty());
        Pipeline::new(self.name, self.shaders)
    }
}

#[derive(Debug, Default)]
pub struct Pipeline {
    pub name: String,
    pub shaders: Vec<Shader>,
}

impl Pipeline {
    pub fn builder<'a>() -> PipelineBuilder<'a> {
        PipelineBuilder::default()
    }

    pub fn new<S: Into<String>>(name: S, reflections: Vec<ShaderReflection>) -> Self {
        let mut shaders = Vec::new();
        assert!(!reflections.is_empty());

        for reflection in reflections {
            shaders.push(Shader::from(reflection));
        }

        Pipeline {
            name: name.into(),
            shaders,
        }
    }
}

impl<'a> From<ShaderReflection<'a>> for Shader {
    fn from(reflection: ShaderReflection) -> Self {
        let entry_point_count = reflection.get_entry_point_count();
        assert_eq!(entry_point_count, 1);
        let entry_point = reflection
            .get_entry_point_by_index(0)
            .expect("Failed to get entry point");
        let stage = entry_point.get_stage();
        let ty = ShaderType::from(stage);

        let mut params = Vec::default();

        let parameter_count = entry_point.get_parameter_count();
        for i in 0..parameter_count {
            let var = entry_point
                .get_parameter_by_index(i)
                .expect("Failed to get parameter by index");
            let var = var.get_variable().unwrap();
            let name = var.get_name();
            let ty = var.get_type().into();
            let param = Param {
                name: name.into(),
                ty,
            };
            params.push(param);
        }

        Shader::new(ty, reflection.path.clone(), params)
    }
}

#[derive(Debug, Default)]
pub struct Shader {
    pub ty: ShaderType,
    /// This is needed for embedding shader input code with include_str!()
    pub path: PathBuf,
    pub params: Vec<Param>,
}

impl Shader {
    pub fn new(ty: ShaderType, path: PathBuf, params: Vec<Param>) -> Self {
        Self { ty, path, params }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum ShaderType {
    #[default]
    Vertex,
    Fragment,
}

impl From<slang::Stage> for ShaderType {
    fn from(stage: slang::Stage) -> Self {
        match stage {
            slang::Stage::Vertex => ShaderType::Vertex,
            slang::Stage::Fragment => ShaderType::Fragment,
            _ => panic!("{}:{}: Unimplemented stage {:?}", file!(), line!(), stage),
        }
    }
}

/// A shader parameter can be any input/output parameter: a vertex attribute,
/// a uniform, a sampler, and so on.
#[derive(Debug, Default)]
pub struct Param {
    name: String,
    ty: ParamType,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ParamType {
    #[default]
    Unknown,
    Vec2,
    Vec3,
    Vec4,
}

impl From<slang::ReflectionType> for ParamType {
    fn from(ty: slang::ReflectionType) -> Self {
        let kind = ty.get_kind();
        let element_count = ty.get_element_count();
        match kind {
            slang::TypeKind::Vector => match element_count {
                2 => Self::Vec2,
                3 => Self::Vec3,
                4 => Self::Vec4,
                _ => Self::Unknown,
            },
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use std::error::Error;

    #[test]
    fn model() {
        let slang = Slang::new();
        let reflections = vec![
            slang.from_path("shaders/simple.vert.slang"),
            slang.from_path("shaders/simple.frag.slang"),
        ];
        let pipeline = Pipeline::new("Test", reflections);
        assert_eq!(pipeline.name, "Test");
        assert_eq!(pipeline.shaders.len(), 2);
        assert_eq!(pipeline.shaders[0].ty, ShaderType::Vertex);
        assert_eq!(pipeline.shaders[1].ty, ShaderType::Fragment);
    }

    #[test]
    fn parse_pipeline() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [shader("vertex")]
            float4 main(float3 pos) : SV_Position {
                return float4(pos, 1.0);
            }
        "#;
        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");
        Ok(())
    }

    #[test]
    fn parse_params() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [shader("vertex")]
            float4 main(float3 pos, float2 uv, float4 color) : SV_Position {
                return float4(pos, 1.0);
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];
        assert_eq!(shader.ty, ShaderType::Vertex);
        assert!(!shader.params.is_empty());
        assert_eq!(shader.params[0].name, "pos");
        assert_eq!(shader.params[0].ty, ParamType::Vec3);
        assert_eq!(shader.params[1].name, "uv");
        assert_eq!(shader.params[1].ty, ParamType::Vec2);
        assert_eq!(shader.params[2].name, "color");
        assert_eq!(shader.params[2].ty, ParamType::Vec4);

        Ok(())
    }
}
