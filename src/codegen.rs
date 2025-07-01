// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use proc_macro2::*;
use quote::*;

use crate::model::*;

impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pipeline_name = format!("Pipeline{}", self.name);
        let pipeline_ident = Ident::new(&pipeline_name, Span::call_site());

        let set_layout_bindings = self.get_set_layout_bindings();

        let vert = &self.shaders[0];
        let frag = &self.shaders[1];

        let vert_path = vert.path.to_string_lossy();
        if !vert.path.exists() {
            panic!(
                "{}:{}: Failed to find `{}`",
                file!(),
                line!(),
                vert.path.display()
            );
        }

        let frag_path = frag.path.to_string_lossy();
        if !frag.path.exists() {
            panic!(
                "{}:{}: Failed to find `{}`",
                file!(),
                line!(),
                frag.path.display()
            );
        }

        let bind_methods = self.get_bind_methods();

        tokens.extend(quote! {
            pub struct #pipeline_ident {
                set_layouts: Vec<vk::DescriptorSetLayout>,
                layout: vk::PipelineLayout,
                pipeline: vk::Pipeline,
                device: std::rc::Rc<ash::Device>,
                name: String,
            }

            impl #pipeline_ident {
                fn create_set_layout(
                    device: &ash::Device,
                    bindings: &[vk::DescriptorSetLayoutBinding],
                ) -> vk::DescriptorSetLayout {
                    let set_layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                        .bindings(bindings);
                    unsafe { device.create_descriptor_set_layout(&set_layout_info, None) }
                        .expect("Failed to create Vulkan descriptor set layout")
                }

                fn new_set_layouts(device: &ash::Device) -> Vec<vk::DescriptorSetLayout> {
                    let set_layout_bindings = [
                        #( #set_layout_bindings, )*
                    ];
                    vec![
                        Self::create_set_layout(device, &set_layout_bindings)
                    ]
                }

                fn new_layout(device: &ash::Device, set_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
                    let create_info = vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(set_layouts);
                    let layout = unsafe { device.create_pipeline_layout(&create_info, None) };
                    layout.expect("Failed to create Vulkan pipeline layout")
                }

                fn new_impl<V: VertexInput>(
                    layout: vk::PipelineLayout,
                    vert_module: &ShaderModule,
                    frag_module: &ShaderModule,
                    pass: vk::RenderPass,
                ) -> vk::Pipeline {
                    let entry = std::ffi::CString::new("main").expect("Failed to create entry point");

                    let stages = [
                        vert_module.get_stage(&entry, vk::ShaderStageFlags::VERTEX),
                        frag_module.get_stage(&entry, vk::ShaderStageFlags::FRAGMENT),
                    ];

                    let vertex_attributes = V::get_attributes();
                    let vertex_bindings = V::get_bindings();

                    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                        .vertex_attribute_descriptions(&vertex_attributes)
                        .vertex_binding_descriptions(&vertex_bindings);

                    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                        .primitive_restart_enable(false);

                    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                        .depth_test_enable(true)
                        .depth_write_enable(true)
                        .depth_compare_op(vk::CompareOp::GREATER)
                        .depth_bounds_test_enable(false)
                        .stencil_test_enable(false);

                    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
                        .line_width(1.0)
                        .depth_clamp_enable(false)
                        .rasterizer_discard_enable(false)
                        .polygon_mode(vk::PolygonMode::FILL)
                        .cull_mode(vk::CullModeFlags::NONE)
                        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                        .depth_bias_enable(false);

                    // Pass as input? Or just use a default value.
                    let width = 480;
                    let height = 480;

                    let viewports = [
                        vk::Viewport::default()
                            .x(0.0)
                            .y(0.0)
                            .width(width as f32)
                            .height(height as f32)
                            .min_depth(0.0)
                            .max_depth(1.0)
                    ];

                    let scissors = [
                        vk::Rect2D::default()
                            .offset(vk::Offset2D::default().x(0).y(0))
                            .extent(vk::Extent2D::default().width(width).height(height))
                    ];

                    let view = vk::PipelineViewportStateCreateInfo::default()
                        .viewports(&viewports)
                        .scissors(&scissors);

                    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                        .sample_shading_enable(false)
                        .alpha_to_coverage_enable(false)
                        .alpha_to_one_enable(false);

                    let blend_attachments = [
                        vk::PipelineColorBlendAttachmentState::default()
                            .blend_enable(false)
                            .color_write_mask(vk::ColorComponentFlags::RGBA)
                    ];

                    let blend = vk::PipelineColorBlendStateCreateInfo::default()
                        .logic_op_enable(false)
                        .attachments(&blend_attachments);

                    let create_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .layout(layout)
                        .render_pass(pass)
                        .subpass(0)
                        .vertex_input_state(&vertex_input)
                        .input_assembly_state(&input_assembly)
                        .depth_stencil_state(&depth_stencil)
                        .rasterization_state(&rasterization)
                        .viewport_state(&view)
                        .multisample_state(&multisample)
                        .color_blend_state(&blend);

                    let pipelines = unsafe { vert_module.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
                    let mut pipelines = pipelines.expect("Failed to create Vulkan graphics pipeline");
                    let pipeline = pipelines.pop().expect("Failed to pop Vulkan pipeline");

                    pipeline
                }

                pub fn new<V: VertexInput>(pass: &Pass) -> Self {
                    let name = String::from(#pipeline_name);

                    let device = pass.device.clone();

                    let set_layouts = Self::new_set_layouts(&device);
                    let layout = Self::new_layout(&device, &set_layouts);

                    let vert_code = SlangProgram::get_entry_point_code(#vert_path, "main").expect("Failed to get code for entry point");
                    let frag_code = SlangProgram::get_entry_point_code(#frag_path, "main").expect("Failed to get code for entry point");

                    let vertex = ShaderModule::from_data(&device, &vert_code);
                    let fragment = ShaderModule::from_data(&device, &frag_code);

                    let pipeline = Self::new_impl::<V>(layout, &vertex, &fragment, pass.render);

                    Self {
                        set_layouts,
                        layout,
                        pipeline,
                        device,
                        name,
                    }
                }

                #( #bind_methods )*
            }

            impl Pipeline for #pipeline_ident {
                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn get_name(&self) -> &String {
                    &self.name
                }

                fn get_set_layouts(&self) -> &[vk::DescriptorSetLayout] {
                    &self.set_layouts
                }

                fn get_layout(&self) -> vk::PipelineLayout {
                    self.layout
                }

                fn get_pipeline(&self) -> vk::Pipeline {
                    self.pipeline
                }
            }

            impl Drop for #pipeline_ident {
                fn drop(&mut self) {
                    unsafe {
                        for set_layout in &self.set_layouts {
                            self.device.destroy_descriptor_set_layout(*set_layout, None);
                        }
                        self.device.destroy_pipeline_layout(self.layout, None);
                        self.device.destroy_pipeline(self.pipeline, None);
                    }
                }
            }
        })
    }
}

pub fn codegen(pipeline: Pipeline) -> TokenStream {
    pipeline.to_token_stream()
}

impl ToTokens for SetLayoutBinding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let binding = self.binding;
        let descriptor_type = self.descriptor_type;
        let stage = self.stage;

        tokens.extend(quote! {
            vk::DescriptorSetLayoutBinding::default()
                .binding(#binding)
                .descriptor_type(#descriptor_type)
                .descriptor_count(1)
                .stage_flags(#stage)
        })
    }
}

impl ToTokens for DescriptorType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            DescriptorType::Uniform => quote! { vk::DescriptorType::UNIFORM_BUFFER },
            DescriptorType::CombinedSampler => {
                quote! { vk::DescriptorType::COMBINED_IMAGE_SAMPLER }
            }
        };
        tokens.extend(new_tokens)
    }
}

impl ToTokens for ShaderType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            ShaderType::Vertex => quote! {vk::ShaderStageFlags::VERTEX},
            ShaderType::Fragment => quote! {vk::ShaderStageFlags::FRAGMENT},
        };
        tokens.extend(new_tokens)
    }
}

impl ToTokens for BindMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Build the signature of the function
        let joined_param_names = self
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<Vec<String>>()
            .join("_and_");

        let bind_signature = format_ident!("bind_{}", joined_param_names);

        // Build the string for the parameters of the function
        let method_params = self.get_method_params();

        tokens.extend(quote! {
            pub fn #bind_signature(
                &self,
                #( #method_params, )*
            ) {

            }
        })
    }
}

impl ToTokens for MethodParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());
        let ty = self.ty;
        tokens.extend(quote! { #name: &#ty })
    }
}

impl ToTokens for ParamType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            ParamType::SampledImage => quote! { Texture },
            _ => quote! { Buffer },
        };
        tokens.extend(new_tokens);
    }
}
