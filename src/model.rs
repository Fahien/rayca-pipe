// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#[derive(Debug, Default)]
pub struct Pipeline {
    pub functions: Vec<Shader>,
}

#[derive(Debug, Default)]
pub struct Shader {}
