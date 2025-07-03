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

        let push_ranges = self.get_push_ranges();
        let set_layouts = self.get_set_layouts();
        let bind_methods = self.get_bind_methods();
        let push_methods = self.get_push_methods();

        tokens.extend(quote! {
            pub struct #pipeline_ident {
                vertex_size: usize,
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
                    vec![
                        #( #set_layouts, )*
                    ]
                }

                fn new_layout(device: &ash::Device, set_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
                    let mut create_info = vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(set_layouts);

                    let push_ranges = [
                        #( #push_ranges, )*
                    ];
                    if !push_ranges.is_empty() {
                        create_info = create_info.push_constant_ranges(&push_ranges);
                    }

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
                        .topology(V::get_topology())
                        .primitive_restart_enable(false);

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
                            .min_depth(1.0)
                            .max_depth(0.0)
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

                    let depth_state = V::get_depth_state();

                    let create_info = vk::GraphicsPipelineCreateInfo::default()
                        .stages(&stages)
                        .layout(layout)
                        .render_pass(pass)
                        .subpass(V::get_subpass())
                        .vertex_input_state(&vertex_input)
                        .input_assembly_state(&input_assembly)
                        .depth_stencil_state(&depth_state)
                        .rasterization_state(&rasterization)
                        .viewport_state(&view)
                        .multisample_state(&multisample)
                        .color_blend_state(&blend);

                    let pipelines = unsafe { vert_module.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None) };
                    let mut pipelines = pipelines.expect("Failed to create Vulkan graphics pipeline");
                    let pipeline = pipelines.pop().expect("Failed to pop Vulkan pipeline");

                    pipeline
                }

                pub fn new<V: VertexInput>(
                    #[cfg(target_os = "android")]
                    android_app: &AndroidApp,
                    pass: &Pass,
                ) -> Self {
                    let name = String::from(#pipeline_name);

                    let device = pass.device.clone();

                    let set_layouts = Self::new_set_layouts(&device);
                    let layout = Self::new_layout(&device, &set_layouts);

                    #[cfg(target_os = "android")]
                    let (vertex, fragment) = ShaderModule::create_shaders(android_app, &device, #vert_path, #frag_path);
                    #[cfg(not(target_os = "android"))]
                    let (vertex, fragment) = ShaderModule::create_shaders(&device, #vert_path, #frag_path);

                    let pipeline = Self::new_impl::<V>(layout, &vertex, &fragment, pass.render);

                    Self {
                        vertex_size: std::mem::size_of::<V>(),
                        set_layouts,
                        layout,
                        pipeline,
                        device,
                        name,
                    }
                }

                #( #bind_methods )*

                #( #push_methods )*
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

                fn get_device(&self) -> &ash::Device {
                    &self.device
                }

                fn get_vertex_size(&self) -> usize {
                    self.vertex_size
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

impl ToTokens for SetLayout {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bindings = &self.bindings;
        tokens.extend(quote! {
            Self::create_set_layout(
                device,
                &[
                    #( #bindings, )*
                ]
            )
        })
    }
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
            DescriptorType::InputAttachment => {
                quote! { vk::DescriptorType::INPUT_ATTACHMENT }
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
            .uniforms
            .iter()
            .map(|uniform| uniform.param.name.clone())
            .collect::<Vec<String>>()
            .join("_and_");

        let bind_signature = format_ident!("bind_{}", joined_param_names);

        // Build the string for the parameters of the function
        let method_params = self.get_method_params();

        let write_sets = self.get_write_sets();

        if self.uniforms.is_empty() {
            panic!(
                "Failed to find uniforms for bind method `{}`",
                bind_signature
            );
        }
        let set = self.uniforms[0].set;

        tokens.extend(quote! {
            pub fn #bind_signature(
                &self,
                command_buffer: &CommandBuffer,
                descriptors: &mut Descriptors,
                key: DescriptorKey,
                #( #method_params, )*
            ) {
                let set_layouts = &[self.get_set_layouts()[#set as usize]];
                let sets = match descriptors.get_or_create(key, set_layouts) {
                    DescriptorEntry::Created(sets) => {
                            unsafe {
                                self.device.update_descriptor_sets(
                                    &[
                                        #( #write_sets, )*
                                    ],
                                    &[]
                                );
                            }
                            sets
                    }
                    DescriptorEntry::Get(sets) => sets,
                };
                command_buffer.bind_descriptor_sets(self.get_layout(), sets, #set);
            }
        })
    }
}

impl ToTokens for MethodParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());
        let ty: VkrType = self.ty.into();
        tokens.extend(quote! { #name: &#ty })
    }
}

impl ToTokens for ParamType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            ParamType::Vec2 => quote! { Vec2 },
            ParamType::Vec3 => quote! { Vec3 },
            ParamType::Vec4 => quote! { Vec4 },
            ParamType::Mat3 => quote! { Mat3 },
            ParamType::Mat4 => quote! { Mat4 },
            _ => panic!("Can not use param type on CPU: `{:?}`", self),
        };
        tokens.extend(new_tokens);
    }
}

impl ToTokens for VkrType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let new_tokens = match self {
            VkrType::Buffer => quote! { Buffer },
            VkrType::Texture => quote! { RenderTexture },
        };
        tokens.extend(new_tokens);
    }
}

impl ToTokens for WriteSet {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let binding = self.binding;
        let descriptor_type = self.descriptor_type;
        let info = &self.info;
        tokens.extend(quote! {
            vk::WriteDescriptorSet::default()
                .dst_set(sets[0])
                .dst_binding(#binding)
                .dst_array_element(0)
                .descriptor_type(#descriptor_type)
        });

        match self.info.ty {
            ParamType::Image | ParamType::SampledImage => {
                tokens.extend(quote! { .image_info(&#info) })
            }
            _ => tokens.extend(quote! { .buffer_info(&#info) }),
        }
        tokens.extend(quote! {});
    }
}

impl ToTokens for WriteSetInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());
        let ty = self.ty;

        match ty {
            ParamType::Image | ParamType::SampledImage => tokens.extend(quote! {
                [
                    vk::DescriptorImageInfo::default()
                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(#name.view)
                        .sampler(#name.sampler)
                ]
            }),
            _ => {
                let size = ty.get_size();
                tokens.extend(quote! {
                    [
                        vk::DescriptorBufferInfo::default()
                            .range(#size as vk::DeviceSize)
                            .buffer(#name.buffer)
                    ]
                });
            }
        }
    }
}

impl ToTokens for PushRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stage = self.stage;
        let range = self.ty.get_size();
        tokens.extend(quote! {
            vk::PushConstantRange::default()
                .offset(0)
                .stage_flags(#stage)
                .size(#range as u32)
        })
    }
}

impl ToTokens for PushMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let push_signature = format_ident!("push_{}", self.name);
        let arg_name = Ident::new(&self.name, Span::call_site());
        let arg_type = &self.ty;
        let stage = self.stage;
        tokens.extend(quote! {
            pub fn #push_signature(&self, command_buffer: &CommandBuffer, #arg_name: &#arg_type) {
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        #arg_name as *const #arg_type as *const u8,
                        std::mem::size_of::<#arg_type>(),
                    )
                };
                command_buffer.push_constants(
                    self,
                    #stage,
                    0, //offset,
                    bytes
                );
            }
        })
    }
}
