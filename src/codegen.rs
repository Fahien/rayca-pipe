// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use proc_macro2::*;
use quote::*;

use crate::Pipeline;

impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pipeline_name = format!("Pipeline{}", self.name);
        let pipeline_ident = Ident::new(&pipeline_name, Span::call_site());

        let vert_path = self.vert_path.to_string_lossy();
        let frag_path = self.frag_path.to_string_lossy();

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

                fn new_impl(vert_module: &ShaderModule, frag_module: &ShaderModule, pass: &Pass) -> vk::Pipeline {
                    let entry = std::ffi::CString::new("main").expect("Failed to create vert entrypoint");

                    let stages = [
                        vert_module.get_stage(&entry, vk::ShaderStageFlags::VERTEX),
                        frag_module.get_stage(&entry, vk::ShaderStageFlags::FRAGMENT),
                    ];

                    let layout = Self::new_layout(&pass.device);

                    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

                    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

                    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
                        .line_width(1.0);

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

                    let states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
                    let dynamics = vk::PipelineDynamicStateCreateInfo::default()
                        .dynamic_states(&states);

                    let create_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .layout(layout)
                        .render_pass(pass.render)
                        .vertex_input_state(&vertex_input)
                        .input_assembly_state(&input_assembly)
                        .rasterization_state(&rasterization)
                        .viewport_state(&view)
                        .multisample_state(&multisample)
                        .dynamic_state(&dynamics);

                    let pipelines = unsafe { pass.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
                    let mut pipelines = pipelines.expect("Failed to create Vulkan graphics pipeline");
                    let pipeline = pipelines.pop().expect("Failed to pop Vulkan pipeline");

                    pipeline
                }

                pub fn new(pass: &Pass) -> Self {
                    let vert_code = SlangProgram::get_entry_point_code(#vert_path, "main").expect("Failed to get code for entry point");
                    let frag_code = SlangProgram::get_entry_point_code(#frag_path, "main").expect("Failed to get code for entry point");

                    let vertex = ShaderModule::from_data(&pass.device, &vert_code);
                    let fragment = ShaderModule::from_data(&pass.device, &frag_code);

                    Self {
                        pipeline: Self::new_impl(&vertex, &fragment, pass)
                    }
                }
            }
        })
    }
}

pub fn codegen(pipeline: Pipeline) -> TokenStream {
    pipeline.to_token_stream()
}
