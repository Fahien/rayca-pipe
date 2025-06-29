// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use proc_macro2::*;
use quote::*;

use crate::Pipeline;

impl ToTokens for Pipeline {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_name = format!("Pipeline{}", self.name);
        let struct_ident = Ident::new(&struct_name, Span::call_site());
        tokens.extend(quote! {
            struct #struct_ident {}
        })
    }
}

pub fn codegen(pipeline: Pipeline) -> TokenStream {
    pipeline.to_token_stream()
}
