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
    fn render(&self, frame: &mut Frame, buffer: &Buffer) {
        self.bind(&frame.cache);
        self.bind_model(&mut frame.cache, buffer);
        self.bind_camera(&mut frame.cache, buffer);
        let texture = Texture::default();
        self.bind_tex_sampler(&mut frame.cache, &texture);
        self.draw(&frame.cache, buffer);
    }
}

impl RenderPipeline for PipelineSecondary {
    fn render(&self, _frame: &mut Frame, _buffer: &Buffer) {}
}

#[test]
fn build_simple_shader() {
    let ctx = Ctx::builder().build();
    let dev = Dev::new(&ctx, None);
    let pass = Pass::new(&dev);
    let _main = PipelineMain::new::<Vertex>(&pass);
    let _secondary = PipelineSecondary::new::<Vertex>(&pass);
}
