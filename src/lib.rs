// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

extern crate proc_macro;

use proc_macro::*;

mod model;
use model::*;
mod parse;
use parse::*;

mod codegen;
use codegen::*;

#[proc_macro]
pub fn pipewriter(input: TokenStream) -> TokenStream {
    let input_string = input.to_string().replace(['\"', ' '], "").replace(',', " ");
    let args: Vec<&str> = input_string.split_whitespace().collect();
    assert_eq!(args.len(), 3);
    let name = args[0];
    let vert_path_str = args[1];
    let frag_path_str = args[2];

    let reflections = [
        ShaderReflection::parse(vert_path_str),
        ShaderReflection::parse(frag_path_str),
    ];
    let pipeline = Pipeline::new(name, &reflections);
    codegen(pipeline).into()
}
