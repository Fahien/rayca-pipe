// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_pipe::pipewriter;

pipewriter!("shaders/simple.vert.slang");

#[test]
fn build_simple_shader() {
    let _p = Pipeline {};
}
