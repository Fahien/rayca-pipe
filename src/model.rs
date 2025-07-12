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

    pub fn get_push_methods(&self) -> Vec<PushMethod> {
        let mut ret = Vec::new();

        for shader in &self.shaders {
            for param in &shader.constants {
                ret.push(PushMethod::new(param.name.clone(), param.ty, shader.ty));
            }
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

            // Guess param type for the moment
            let param_type = ParamType::from_type(ty);

            match category {
                slang::ParameterCategory::VaryingInput => {
                    let param = Param::new(name.into(), param_type);
                    params.push(param);
                }
                slang::ParameterCategory::PushConstantBuffer => {
                    let param = Param::new(name.into(), param_type);
                    constants.push(param);
                }
                slang::ParameterCategory::Uniform | slang::ParameterCategory::Subpass => {
                    let binding = var_layout.get_binding_index();
                    let set = var_layout.get_binding_space();
                    let param = Param::new(name.into(), param_type);
                    let uniform = Uniform::new(param, set, binding, 0);
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

            // Guess param type for the moment
            let mut param_type = ParamType::from_type(ty);

            match category {
                slang::ParameterCategory::PushConstantBuffer => {
                    let param = Param::new(name.into(), param_type);
                    constants.push(param)
                }
                slang::ParameterCategory::DescriptorTableSlot
                | slang::ParameterCategory::Uniform => {
                    let binding = var_layout.get_binding_index();
                    let set = var_layout.get_binding_space();
                    let param = Param::new(name.into(), param_type);
                    let uniform = Uniform::new(param, set, binding, 0);
                    uniforms.push(uniform)
                }
                slang::ParameterCategory::Mixed => {
                    let mut set = 0;
                    let mut binding = 0;
                    let mut input_attachment_index = 0;

                    for i in 0..type_layout.get_category_count() {
                        let sub_category = type_layout.get_category_by_index(i);
                        match sub_category {
                            slang::ParameterCategory::DescriptorTableSlot => {
                                binding = var_layout.get_offset(sub_category) as u32;
                                set = var_layout.get_binding_space_for_category(sub_category);
                            }
                            slang::ParameterCategory::Subpass => {
                                // This is a subpass input, so the param type should be `Image`
                                param_type = ParamType::Image;
                                input_attachment_index = var_layout.get_offset(sub_category) as u32;
                            }
                            _ => panic!(
                                "{}:{}: unsupported sub category {:?}",
                                file!(),
                                line!(),
                                sub_category
                            ),
                        }
                    }

                    let param = Param::new(name.into(), param_type);
                    uniforms.push(Uniform::new(param, set, binding, input_attachment_index))
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

impl Param {
    pub fn new(name: String, ty: ParamType) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone, Debug)]
pub struct Uniform {
    pub param: Param,
    pub set: u32,
    binding: u32,
    input_attachment_index: u32,
}

impl Uniform {
    pub fn new(param: Param, set: u32, binding: u32, input_attachment_index: u32) -> Self {
        Self {
            param,
            set,
            binding,
            input_attachment_index,
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

impl ParamType {
    fn get_type_size(ty: slang::ReflectionType) -> usize {
        let kind = ty.get_kind();
        let element_count = ty.get_element_count();
        let column_count = ty.get_column_count();
        let row_count = ty.get_row_count();
        match kind {
            slang::TypeKind::Vector => match element_count {
                2 => 2 * 4,
                3 => 3 * 4,
                4 => 4 * 4,
                _ => panic!(
                    "{}:{}: unsupported vector[{}]",
                    file!(),
                    line!(),
                    element_count
                ),
            },
            slang::TypeKind::Matrix => match (row_count, column_count) {
                (3, 3) => 3 * 3 * 4,
                (4, 4) => 4 * 4 * 4,
                _ => panic!(
                    "{}:{}: unsupported matrix[{}][{}]",
                    file!(),
                    line!(),
                    row_count,
                    column_count
                ),
            },
            slang::TypeKind::ConstantBuffer => {
                let element_type = ty.get_element_type().unwrap();
                Self::get_type_size(element_type)
            }
            slang::TypeKind::Struct => {
                let mut size = 0;
                for i in 0..ty.get_field_count() {
                    if let Some(field) = ty.get_field_by_index(i) {
                        let field_ty = field.get_type();
                        size += ParamType::get_type_size(field_ty);
                    }
                }
                size
            }
            _ => panic!("{}:{}: unsupported slang type {:?}", file!(), line!(), kind),
        }
    }

    fn from_type(ty: slang::ReflectionType) -> Self {
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
            slang::TypeKind::ConstantBuffer => {
                let element_type = ty.get_element_type().unwrap();
                Self::from_type(element_type)
            }
            slang::TypeKind::Resource => Self::SampledImage,
            slang::TypeKind::Struct => {
                let size = Self::get_type_size(ty);
                // Align size to 16 bytes
                let size = (size + 15) / 16 * 16;
                Self::Struct(size)
            }
            slang::TypeKind::SamplerState => Self::SampledImage,
            _ => panic!("{}:{}: unsupported slang type {:?}", file!(), line!(), kind),
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            ParamType::Vec2 => std::mem::size_of::<f32>() * 2,
            ParamType::Vec3 => std::mem::size_of::<f32>() * 4, // simd
            ParamType::Vec4 => std::mem::size_of::<f32>() * 4,
            ParamType::Mat3 => std::mem::size_of::<f32>() * 9,
            ParamType::Mat4 => std::mem::size_of::<f32>() * 16,
            ParamType::Struct(size) => {
                if *size == 0 {
                    panic!("{}:{}: Struct size is not known", file!(), line!());
                }
                *size
            }
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

/// Methods for pushing constants
#[derive(Clone, Debug)]
pub struct PushMethod {
    pub name: String,
    pub ty: ParamType,
    pub stage: ShaderType,
}

impl PushMethod {
    pub fn new(name: String, ty: ParamType, stage: ShaderType) -> Self {
        Self { name, ty, stage }
    }
}

#[derive(Copy, Clone)]
pub enum VkrType {
    Buffer,
    Texture,
}

impl From<ParamType> for VkrType {
    fn from(ty: ParamType) -> Self {
        match ty {
            ParamType::SampledImage | ParamType::Image => Self::Texture,
            _ => Self::Buffer,
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
            layout (input_attachment_index = 1, set = 2, binding = 3)
            SubpassInput scene_color;

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

        assert_eq!(shader.uniforms[0].param.ty, ParamType::Image);
        assert_eq!(shader.uniforms[0].input_attachment_index, 1);
        assert_eq!(shader.uniforms[0].set, 2);
        assert_eq!(shader.uniforms[0].binding, 3);

        Ok(())
    }

    #[test]
    fn parse_constants() -> Result<(), Box<dyn Error>> {
        let code = r#"
            [vk::push_constant] float4 color;
            [shader("fragment")]
            float4 main() : SV_Target {
                return color;
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];
        assert_eq!(shader.constants[0].ty, ParamType::Vec4);

        Ok(())
    }

    #[test]
    fn parse_complex_constants() -> Result<(), Box<dyn Error>> {
        let code = r#"
            struct PushConstants {
                float4 color;
            };
            [vk::push_constant] PushConstants constants;
            [shader("fragment")]
            float4 main() : SV_Target {
                return constants.color;
            }
        "#;

        let slang = Slang::new();
        let vert = slang.from_source("test", code);
        let pipeline = Pipeline::builder().name("Shader").vert(vert).build();
        assert_eq!(pipeline.name, "Shader");

        assert!(!pipeline.shaders.is_empty());
        let shader = &pipeline.shaders[0];
        assert_eq!(shader.constants[0].ty, ParamType::Struct(16));

        Ok(())
    }
}
