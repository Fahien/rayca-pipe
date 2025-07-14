// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_core::*;
use rayca_pipe::*;

pipewriter!(
    Main,
    "shaders/simple.vert.slang",
    "shaders/simple.frag.slang"
);
pipewriter!(
    Secondary,
    "shaders/simple.vert.slang",
    "shaders/simple.frag.slang"
);

impl RenderPipeline for PipelineMain {
    fn render(
        &self,
        frame: &mut Frame,
        scene: &RenderScene,
        _camera_infos: &[CameraDrawInfo],
        _infos: Vec<DrawInfo>,
    ) {
        let model = scene.get_default_model();

        self.bind(&frame.cache);

        let buffer = &model.primitives[0].vertices;

        let key = DescriptorKey::default();

        self.bind_model(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            buffer,
        );
        self.bind_view_and_proj(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            buffer,
            buffer,
        );
        let texture = RenderTexture::default();
        self.bind_color_and_albedo(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            buffer,
            &texture,
        );

        self.bind_scene_color(
            &frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            key,
            &texture,
        );

        let constants = PushConstants::default();
        self.push_constants(&frame.cache.command_buffer, &constants);

        self.draw(&frame.cache, &model.primitives[0]);
    }
}

#[derive(Default)]
struct PushConstants {
    _pretransform: Mat4,
}

impl AsBytes for PushConstants {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl RenderPipeline for PipelineSecondary {
    fn render(
        &self,
        _frame: &mut Frame,
        _scene: &RenderScene,
        _camera_infos: &[CameraDrawInfo],
        _infos: Vec<DrawInfo>,
    ) {
    }
}

#[test]
fn build_simple_shader() {
    let ctx = Ctx::builder().build();
    let dev = Dev::new(&ctx, None);
    let pass = Pass::new(&dev);
    let _main = PipelineMain::new::<Vertex>(&pass);
    let _secondary = PipelineSecondary::new::<Vertex>(&pass);
}
