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

    pub fn get_set_layout_bindings(&self) -> Vec<SetLayoutBinding> {
        let mut ret = Vec::new();
        ret.extend(self.shaders[0].get_set_layout_bindings());
        ret.extend(self.shaders[1].get_set_layout_bindings());
        ret
    }

    pub fn get_bind_methods(&self) -> Vec<BindMethod> {
        let mut ret = Vec::new();
        ret.extend(self.shaders[0].get_bind_methods());
        ret.extend(self.shaders[1].get_bind_methods());
        ret
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
        let mut uniforms = Vec::default();

        let parameter_count = entry_point.get_parameter_count();
        for i in 0..parameter_count {
            let var_layout = entry_point
                .get_parameter_by_index(i)
                .expect("Failed to get parameter by index");
            let var = var_layout.get_variable().unwrap();
            let name = var.get_name();
            let ty = var.get_type();
            let type_layout = var_layout.get_type_layout().unwrap();
            let category = type_layout.get_parameter_category();

            let param = Param {
                name: name.into(),
                ty: ty.into(),
            };

            match category {
                slang::ParameterCategory::VaryingInput => params.push(param),
                slang::ParameterCategory::Uniform => {
                    let binding = var_layout.get_binding_index();
                    let set = var_layout.get_binding_space();
                    let uniform = Uniform::new(param, set, binding);
                    uniforms.push(uniform)
                }
                _ => panic!(
                    "{}:{}: Unimplemented category `{:?}`",
                    file!(),
                    line!(),
                    category
                ),
            }
        }

        let parameter_count = reflection.get_parameter_count();
        for i in 0..parameter_count {
            let var_layout = reflection
                .get_parameter_by_index(i)
                .expect("Failed to get parameter by index");
            let var = var_layout.get_variable().unwrap();
            let name = var.get_name();
            let ty = var.get_type();
            let type_layout = var_layout.get_type_layout().unwrap();
            let category = type_layout.get_parameter_category();

            let param = Param {
                name: name.into(),
                ty: ty.into(),
            };

            match category {
                slang::ParameterCategory::DescriptorTableSlot
                | slang::ParameterCategory::Uniform => {
                    let binding = var_layout.get_binding_index();
                    let set = var_layout.get_binding_space();
                    let uniform = Uniform::new(param, set, binding);
                    uniforms.push(uniform)
                }
                _ => panic!(
                    "{}:{}: Unimplemented category `{:?}`",
                    file!(),
                    line!(),
                    category
                ),
            }
        }

        Shader::new(ty, reflection.path.clone(), params, uniforms)
    }
}

#[derive(Debug, Default)]
pub struct Shader {
    pub ty: ShaderType,
    /// This is needed for embedding shader input code with include_str!()
    pub path: PathBuf,
    pub params: Vec<Param>,
    pub uniforms: Vec<Uniform>,
}

impl Shader {
    pub fn new(ty: ShaderType, path: PathBuf, params: Vec<Param>, uniforms: Vec<Uniform>) -> Self {
        Self {
            ty,
            path,
            params,
            uniforms,
        }
    }

    pub fn get_set_layout_bindings(&self) -> Vec<SetLayoutBinding> {
        let mut ret = Vec::new();
        for param in &self.uniforms {
            ret.push(param.get_set_layout_binding(self.ty));
        }
        ret
    }

    pub fn get_bind_methods(&self) -> Vec<BindMethod> {
        let mut ret = Vec::new();
        for uniform in &self.uniforms {
            ret.push(uniform.get_bind_method());
        }
        ret
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
#[derive(Debug)]
pub struct Param {
    name: String,
    ty: ParamType,
}

#[derive(Debug)]
pub struct Uniform {
    param: Param,
    set: u32,
    binding: u32,
}

impl Uniform {
    pub fn new(param: Param, set: u32, binding: u32) -> Self {
        Self {
            param,
            set,
            binding,
        }
    }

    pub fn get_set_layout_binding(&self, stage: ShaderType) -> SetLayoutBinding {
        SetLayoutBinding {
            stage,
            descriptor_type: self.param.ty.into(),
            descriptor_set: self.set,
            binding: self.binding,
        }
    }

    pub fn get_bind_method(&self) -> BindMethod {
        BindMethod {
            name: self.param.name.clone().to_lowercase(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ParamType {
    #[default]
    Unknown,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
}

impl From<slang::ReflectionType> for ParamType {
    fn from(ty: slang::ReflectionType) -> Self {
        let kind = ty.get_kind();
        let element_count = ty.get_element_count();
        let column_count = ty.get_column_count();
        let row_count = ty.get_row_count();
        match kind {
            slang::TypeKind::Vector => match element_count {
                2 => Self::Vec2,
                3 => Self::Vec3,
                4 => Self::Vec4,
                _ => Self::Unknown,
            },
            slang::TypeKind::Matrix => match (row_count, column_count) {
                (4, 4) => Self::Mat4,
                _ => Self::Unknown,
            },
            slang::TypeKind::ConstantBuffer => ty.get_element_type().unwrap().into(),
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DescriptorType {
    Uniform,
    CombinedSampler,
}

impl From<ParamType> for DescriptorType {
    fn from(param: ParamType) -> Self {
        match param {
            ParamType::Unknown => panic!(
                "{}:{}: No descriptor type for `{:?}`",
                file!(),
                line!(),
                param
            ),
            ParamType::Vec2 | ParamType::Vec3 | ParamType::Vec4 | ParamType::Mat4 => {
                DescriptorType::Uniform
            }
        }
    }
}

pub struct SetLayoutBinding {
    pub stage: ShaderType,
    pub descriptor_type: DescriptorType,
    pub descriptor_set: u32,
    pub binding: u32,
}

pub struct BindMethod {
    pub name: String,
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
        assert_eq!(shader.params.len(), 3);
        assert_eq!(shader.params[0].name, "pos");
        assert_eq!(shader.params[0].ty, ParamType::Vec3);
        assert_eq!(shader.params[1].name, "uv");
        assert_eq!(shader.params[1].ty, ParamType::Vec2);
        assert_eq!(shader.params[2].name, "color");
        assert_eq!(shader.params[2].ty, ParamType::Vec4);

        Ok(())
    }

    #[test]
    fn parse_uniforms() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [vk::binding(0, 0)]
            ConstantBuffer<float4x4> model;

            [vk::binding(0, 1)]
            ConstantBuffer<float4x4> view_proj;

            [shader("vertex")]
            float4 main(
                float3 pos,
                float2 uv,
                float4 color,
            ) : SV_Position {
                return mul(view_proj, mul(model, float4(pos, 1.0)));
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];
        assert_eq!(shader.ty, ShaderType::Vertex);
        assert_eq!(shader.params.len(), 3);
        assert_eq!(shader.params[0].name, "pos");
        assert_eq!(shader.params[0].ty, ParamType::Vec3);
        assert_eq!(shader.params[1].name, "uv");
        assert_eq!(shader.params[1].ty, ParamType::Vec2);
        assert_eq!(shader.params[2].name, "color");
        assert_eq!(shader.params[2].ty, ParamType::Vec4);
        assert_eq!(shader.uniforms[0].param.name, "model");
        assert_eq!(shader.uniforms[0].param.ty, ParamType::Mat4);
        assert_eq!(shader.uniforms[0].set, 0);
        assert_eq!(shader.uniforms[0].binding, 0);
        assert_eq!(shader.uniforms[1].param.name, "view_proj");
        assert_eq!(shader.uniforms[1].param.ty, ParamType::Mat4);
        assert_eq!(shader.uniforms[1].set, 1);
        assert_eq!(shader.uniforms[1].binding, 0);

        Ok(())
    }
}
