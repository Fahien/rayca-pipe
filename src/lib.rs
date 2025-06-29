// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

extern crate proc_macro;

use proc_macro::*;

use quote::quote;

mod model;
use model::*;
mod parse;
use parse::*;

#[proc_macro]
pub fn pipewriter(input: TokenStream) -> TokenStream {
    let file_path_str = input.to_string().replace('\"', "");
    let pipeline = parse_shader(&file_path_str);
    codegen(pipeline)
}

fn codegen(pipeline: Pipeline) -> TokenStream {
    eprintln!("{:?}", pipeline);

    quote! {
        struct Pipeline {

        }
    }
    .into()
}
