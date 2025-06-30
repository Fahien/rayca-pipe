// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use proc_macro2::*;
use quote::*;

use crate::Pipeline;

impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pipeline_name = format!("Pipeline{}", self.name);
        let pipeline_ident = Ident::new(&pipeline_name, Span::call_site());

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

        tokens.extend(quote! {
            pub struct #pipeline_ident {
                pipeline: vk::Pipeline,
            }

            impl #pipeline_ident {
                pub fn new_layout(device: &ash::Device) -> vk::PipelineLayout {
                    let create_info = vk::PipelineLayoutCreateInfo::default();
                    let layout = unsafe { device.create_pipeline_layout(&create_info, None) };
                    layout.expect("Failed to create Vulkan pipeline layout")
                }

                fn new_impl<V: VertexInput>(vert_module: &ShaderModule, frag_module: &ShaderModule, pass: &Pass) -> vk::Pipeline {
                    let entry = std::ffi::CString::new("main").expect("Failed to create entry point");

                    let stages = [
                        vert_module.get_stage(&entry, vk::ShaderStageFlags::VERTEX),
                        frag_module.get_stage(&entry, vk::ShaderStageFlags::FRAGMENT),
                    ];

                    let layout = Self::new_layout(&pass.device);

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
                    let width = 1920;
                    let height = 1080;

                    let viewports = [
                        vk::Viewport::default()
                            .x(0.0)
                            .y(0.0)
                            .width(width as f32)
                            .height(height as f32)
                            .min_depth(1.0) // TODO: 1.0 is near?
                            .max_depth(0.0) // 0.0 is far?
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
                        .blend_enable(true)
                        .color_write_mask(
                            vk::ColorComponentFlags::R
                                | vk::ColorComponentFlags::G
                                | vk::ColorComponentFlags::B,
                        )
                        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                        .color_blend_op(vk::BlendOp::ADD)
                        .src_alpha_blend_factor(vk::BlendFactor::ONE)
                        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                        .color_blend_op(vk::BlendOp::ADD)
                    ];

                    let blend = vk::PipelineColorBlendStateCreateInfo::default()
                        .logic_op_enable(false)
                        .attachments(&blend_attachments);

                    let states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
                    let dynamics = vk::PipelineDynamicStateCreateInfo::default()
                        .dynamic_states(&states);

                    let create_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .layout(layout)
                        .render_pass(pass.render)
                        .subpass(0)
                        .vertex_input_state(&vertex_input)
                        .input_assembly_state(&input_assembly)
                        .depth_stencil_state(&depth_stencil)
                        .rasterization_state(&rasterization)
                        .viewport_state(&view)
                        .multisample_state(&multisample)
                        .color_blend_state(&blend)
                        .dynamic_state(&dynamics);

                    let pipelines = unsafe { pass.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
                    let mut pipelines = pipelines.expect("Failed to create Vulkan graphics pipeline");
                    let pipeline = pipelines.pop().expect("Failed to pop Vulkan pipeline");

                    pipeline
                }

                pub fn new<V: VertexInput>(pass: &Pass) -> Self {
                    let vert_code = SlangProgram::get_entry_point_code(#vert_path, "main").expect("Failed to get code for entry point");
                    let frag_code = SlangProgram::get_entry_point_code(#frag_path, "main").expect("Failed to get code for entry point");

                    let vertex = ShaderModule::from_data(&pass.device, &vert_code);
                    let fragment = ShaderModule::from_data(&pass.device, &frag_code);

                    Self {
                        pipeline: Self::new_impl::<V>(&vertex, &fragment, pass)
                    }
                }
            }
        })
    }
}

pub fn codegen(pipeline: Pipeline) -> TokenStream {
    pipeline.to_token_stream()
}
