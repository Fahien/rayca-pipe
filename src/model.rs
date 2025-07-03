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

    pub fn get_set_layouts(&self) -> Vec<SetLayout> {
        let mut ret = Vec::new();

        let vert = &self.shaders[0];
        let frag = &self.shaders[1];

        if vert.uniforms.is_empty() && frag.uniforms.is_empty() {
            return ret;
        }

        // Find the number of descriptor looking into both shaders
        let descriptor_count = vert.get_descriptor_max().max(frag.get_descriptor_max()) + 1;
        for set in 0..descriptor_count {
            ret.push(SetLayout::new(self.get_set_layout_bindings(set)));
        }

        ret
    }

    pub fn get_set_layout_bindings(&self, set: u32) -> Vec<SetLayoutBinding> {
        let mut ret = Vec::new();
        let vert = &self.shaders[0];
        let frag = &self.shaders[1];
        ret.extend(vert.get_set_layout_bindings(set));
        ret.extend(frag.get_set_layout_bindings(set));
        ret
    }

    pub fn get_bind_methods(&self) -> Vec<BindMethod> {
        let mut ret = Vec::new();

        let vert = &self.shaders[0];
        let frag = &self.shaders[1];

        if vert.uniforms.is_empty() && frag.uniforms.is_empty() {
            return ret;
        }

        // Find the number of descriptor looking into both shaders
        let descriptor_count = vert.get_descriptor_max().max(frag.get_descriptor_max()) + 1;
        ret.resize(descriptor_count as usize, BindMethod::default());

        vert.get_bind_methods(&mut ret);
        frag.get_bind_methods(&mut ret);
        ret
    }

    pub fn get_push_ranges(&self) -> Vec<PushRange> {
        let mut ret = Vec::new();
        for param in &self.shaders[0].constants {
            ret.push(PushRange::new(param.ty, ShaderType::Vertex));
        }
        for param in &self.shaders[1].constants {
            ret.push(PushRange::new(param.ty, ShaderType::Fragment));
        }
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
        let mut constants = Vec::default();

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
                slang::ParameterCategory::PushConstantBuffer => constants.push(param),
                slang::ParameterCategory::Uniform | slang::ParameterCategory::Subpass => {
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
                slang::ParameterCategory::PushConstantBuffer => constants.push(param),
                slang::ParameterCategory::DescriptorTableSlot
                | slang::ParameterCategory::Uniform
                | slang::ParameterCategory::Mixed => {
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

        // Remove samplers from uniforms and store them in another vector
        let mut samplers = Vec::new();
        uniforms.retain(|uniform| {
            if uniform.param.ty == ParamType::SampledImage {
                samplers.push(uniform.clone());
                false
            } else {
                true
            }
        });

        for sampler in samplers {
            if uniforms
                .iter()
                .any(|u| u.set == sampler.set && u.binding == sampler.binding)
            {
                // If a uniform with the same set and binding already exists, skip this sampler
                continue;
            } else {
                // Otherwise, add the sampler as a uniform
                uniforms.push(sampler);
            }
        }

        uniforms.sort_by_key(|uniform| uniform.binding);
        Shader::new(ty, reflection.path.clone(), params, uniforms, constants)
    }
}

#[derive(Debug, Default)]
pub struct Shader {
    pub ty: ShaderType,
    /// This is needed for embedding shader input code with include_str!()
    pub path: PathBuf,
    pub params: Vec<Param>,
    pub uniforms: Vec<Uniform>,
    pub constants: Vec<Param>,
}

impl Shader {
    pub fn new(
        ty: ShaderType,
        path: PathBuf,
        params: Vec<Param>,
        uniforms: Vec<Uniform>,
        constants: Vec<Param>,
    ) -> Self {
        Self {
            ty,
            path,
            params,
            uniforms,
            constants,
        }
    }

    pub fn get_set_layout_bindings(&self, set: u32) -> Vec<SetLayoutBinding> {
        let mut ret = Vec::new();
        for uniform in &self.uniforms {
            if uniform.set == set {
                ret.push(uniform.get_set_layout_binding(self.ty));
            }
        }
        ret
    }

    pub fn get_descriptor_max(&self) -> u32 {
        let mut descriptor_max = 0;
        for uniform in &self.uniforms {
            descriptor_max = descriptor_max.max(uniform.set);
        }
        descriptor_max
    }

    pub fn get_bind_methods(&self, methods: &mut [BindMethod]) {
        for uniform in &self.uniforms {
            methods[uniform.set as usize].uniforms.push(uniform.clone());
        }
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
#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    ty: ParamType,
}

#[derive(Clone, Debug)]
pub struct Uniform {
    pub param: Param,
    pub set: u32,
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
            binding: self.binding,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamType {
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    SampledImage,
    Image,
    Sampler,
    Struct(usize),
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
                _ => panic!(
                    "{}:{}: unsupported vector[{}]",
                    file!(),
                    line!(),
                    element_count
                ),
            },
            slang::TypeKind::Matrix => match (row_count, column_count) {
                (3, 3) => Self::Mat3,
                (4, 4) => Self::Mat4,
                _ => panic!(
                    "{}:{}: unsupported matrix[{}][{}]",
                    file!(),
                    line!(),
                    row_count,
                    column_count
                ),
            },
            slang::TypeKind::ConstantBuffer => ty.get_element_type().unwrap().into(),
            slang::TypeKind::Resource => Self::SampledImage,
            slang::TypeKind::Struct => Self::Struct(0),
            slang::TypeKind::SamplerState => Self::SampledImage,
            _ => panic!("{}:{}: unsupported slang type {:?}", file!(), line!(), kind),
        }
    }
}
impl ParamType {
    pub fn get_size(&self) -> usize {
        match self {
            ParamType::Vec2 => std::mem::size_of::<f32>() * 2,
            ParamType::Vec3 => std::mem::size_of::<f32>() * 4, // simd
            ParamType::Vec4 => std::mem::size_of::<f32>() * 4,
            ParamType::Mat3 => std::mem::size_of::<f32>() * 9,
            ParamType::Mat4 => std::mem::size_of::<f32>() * 16,
            ParamType::Struct(size) => *size,
            _ => panic!("{}:{}: no size for `{:?}`", file!(), line!(), self),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DescriptorType {
    Uniform,
    CombinedSampler,
    InputAttachment,
}

impl From<ParamType> for DescriptorType {
    fn from(param: ParamType) -> Self {
        match param {
            ParamType::SampledImage => DescriptorType::CombinedSampler,
            ParamType::Image => DescriptorType::InputAttachment,
            _ => DescriptorType::Uniform,
        }
    }
}

#[derive(Default)]
pub struct SetLayout {
    pub bindings: Vec<SetLayoutBinding>,
}

impl SetLayout {
    pub fn new(bindings: Vec<SetLayoutBinding>) -> SetLayout {
        Self { bindings }
    }
}

pub struct SetLayoutBinding {
    pub stage: ShaderType,
    pub descriptor_type: DescriptorType,
    pub binding: u32,
}

#[derive(Clone, Default, Debug)]
pub struct BindMethod {
    pub uniforms: Vec<Uniform>,
}

impl BindMethod {
    pub fn get_method_params(&self) -> Vec<MethodParam> {
        let mut ret = Vec::new();
        for uniform in &self.uniforms {
            ret.push(MethodParam {
                name: uniform.param.name.clone(),
                ty: uniform.param.ty,
            })
        }
        ret
    }

    pub fn get_write_sets(&self) -> Vec<WriteSet> {
        let mut ret = Vec::new();
        for uniform in &self.uniforms {
            ret.push(WriteSet {
                binding: uniform.binding,
                descriptor_type: uniform.param.ty.into(),
                info: WriteSetInfo {
                    name: uniform.param.name.clone(),
                    ty: uniform.param.ty,
                },
            })
        }
        ret
    }
}

#[derive(Clone, Debug)]
pub struct MethodParam {
    pub name: String,
    pub ty: ParamType,
}

#[derive(Clone, Debug)]
pub struct WriteSet {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub info: WriteSetInfo,
}

/// The info associated to the `WriteDescriptorSet` changes according to the
/// type of the parameter.
#[derive(Clone, Debug)]
pub struct WriteSetInfo {
    pub name: String,
    pub ty: ParamType,
}

/// Push constant range for constructing the pipeline layout
#[derive(Clone, Debug)]
pub struct PushRange {
    pub ty: ParamType,
    pub stage: ShaderType,
}

impl PushRange {
    pub fn new(ty: ParamType, stage: ShaderType) -> Self {
        Self { ty, stage }
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

    #[test]
    fn parse_multiple_uniforms() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [vk::binding(0)]
            ConstantBuffer<float4> color;

            [vk::binding(1)]
            Texture2D tex_sampler;

            [shader("fragment")]
            float4 main(
            ) : SV_Target {
                return color;
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];

        assert_eq!(shader.uniforms[0].param.ty, ParamType::Vec4);
        assert_eq!(shader.uniforms[1].param.ty, ParamType::SampledImage);

        Ok(())
    }

    #[test]
    fn parse_input_attachment() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [[vk::input_attachment_index(0)]] SubpassInput scene_color;
            [shader("fragment")]
            float4 main() : SV_Target {
                return scene_color.SubpassLoad();
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];

        Ok(())
    }
}
