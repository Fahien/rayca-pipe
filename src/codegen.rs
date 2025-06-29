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

                pub fn new(device: &std::rc::Rc<ash::Device>) -> Self {
                    let vert_code = SlangProgram::get_entry_point_code(#vert_path, "main").expect("Failed to get code for entry point");
                    let frag_code = SlangProgram::get_entry_point_code(#frag_path, "main").expect("Failed to get code for entry point");

                    let vert_module = ShaderModule::from_data(device, &vert_code);
                    let frag_module = ShaderModule::from_data(device, &frag_code);

                    let vert_entry = std::ffi::CString::new("vert_main").expect("Failed to create vert entrypoint");
                    let frag_entry = std::ffi::CString::new("frag_main").expect("Failed to create frag entrypoint");


                    let stages = [
                        vert_module.get_stage(&vert_entry, vk::ShaderStageFlags::VERTEX),
                        frag_module.get_stage(&frag_entry, vk::ShaderStageFlags::FRAGMENT)
                    ];

                    let layout = Self::new_layout(device);

                    let create_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .layout(layout);

                    let pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
                    let mut pipelines = pipelines.expect("Failed to create Vulkan graphics pipeline");
                    let pipeline = pipelines.pop().expect("Failed to pop Vulkan pipeline");

                    Self {
                        pipeline
                    }
                }
            }
        })
    }
}

pub fn codegen(pipeline: Pipeline) -> TokenStream {
    pipeline.to_token_stream()
}
