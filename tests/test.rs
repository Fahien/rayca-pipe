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

#[test]
fn build_simple_shader() {
    let ctx = Ctx::builder().build();
    let dev = Dev::new(&ctx, None);
    let pass = Pass::new(&dev);
    let _main = PipelineMain::new(&pass);
    let _secondary = PipelineSecondary::new(&pass);
}
