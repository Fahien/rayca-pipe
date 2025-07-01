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
/// Takes as input:
/// - the name of the pipeline
/// - a search path
/// - a path to a vertex shader
/// - a path to a fragment shader
pub fn pipewriter(input: TokenStream) -> TokenStream {
    let input_string = input.to_string().replace(['\"', ' '], "").replace(',', " ");
    let args: Vec<&str> = input_string.split_whitespace().collect();
    assert_eq!(args.len(), 3);
    let name = args[0];
    let vert_path = args[1];
    let frag_path = args[2];

    let slang = Slang::new();
    let vert = slang.from_path(&vert_path);
    let frag = slang.from_path(&frag_path);

    let pipeline = Pipeline::builder().name(name).vert(vert).frag(frag).build();
    codegen(pipeline).into()
}
