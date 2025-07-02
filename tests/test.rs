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
    fn render(&self, frame: &mut Frame, model: &RenderModel, nodes: &[Handle<Node>]) {
        self.bind(&frame.cache);

        let buffer = &model.primitives[0].vertices;
        let node = nodes[0];

        self.bind_model(
            frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            node,
            buffer,
        );
        self.bind_camera(
            frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            node,
            buffer,
        );
        let texture = RenderTexture::default();
        self.bind_color_and_albedo(
            frame.cache.command_buffer,
            &mut frame.cache.descriptors,
            node,
            buffer,
            &texture,
        );
        self.draw(&frame.cache, &model.primitives[0]);
    }
}

impl RenderPipeline for PipelineSecondary {
    fn render(&self, _frame: &mut Frame, _model: &RenderModel, _nodes: &[Handle<Node>]) {}
}

#[test]
fn build_simple_shader() {
    let ctx = Ctx::builder().build();
    let dev = Dev::new(&ctx, None);
    let pass = Pass::new(&dev);
    let _main = PipelineMain::new::<Vertex>(&pass);
    let _secondary = PipelineSecondary::new::<Vertex>(&pass);
}
