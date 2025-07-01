// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::marker::PhantomData;
use std::path::PathBuf;

use slang::Downcast;

pub struct Slang {
    session: slang::Session,
    _global_session: slang::GlobalSession,
}

impl Slang {
    pub fn new() -> Slang {
        let global_session = slang::GlobalSession::new().unwrap();

        // All compiler options are available through this builder.
        let session_options = slang::CompilerOptions::default()
            .optimization(slang::OptimizationLevel::High)
            .matrix_layout_row(true);

        let targets = [slang::TargetDesc::default()
            .format(slang::CompileTarget::Spirv)
            .profile(global_session.find_profile("sm_6_5"))];

        let session_desc = slang::SessionDesc::default()
            .targets(&targets)
            .options(&session_options);

        let session = global_session.create_session(&session_desc).unwrap();

        Slang {
            session,
            _global_session: global_session,
        }
    }

    pub fn from_path<'a, P: Into<PathBuf>>(&'a self, path: P) -> ShaderReflection<'a> {
        ShaderReflection::from_path(self, path)
    }

    #[allow(unused)]
    pub fn from_source<'a, P: Into<PathBuf>, S: Into<String>>(
        &'a self,
        path: P,
        source: S,
    ) -> ShaderReflection<'a> {
        ShaderReflection::from_source(self, path, source)
    }
}

pub struct ShaderReflection<'a> {
    pub path: PathBuf,
    reflection: slang::ShaderReflection,
    _program: slang::ComponentType,
    _module: slang::Module,
    _phantom: PhantomData<&'a i32>,
}

impl<'a> std::ops::Deref for ShaderReflection<'a> {
    type Target = slang::ShaderReflection;

    fn deref(&self) -> &Self::Target {
        &self.reflection
    }
}

impl<'a> ShaderReflection<'a> {
    #[allow(unused)]
    pub fn from_source<P: Into<PathBuf>, S: Into<String>>(
        slang: &'a Slang,
        path: P,
        source: S,
    ) -> ShaderReflection<'a> {
        let path = path.into();
        let name = path.to_string_lossy();
        let source = source.into();

        let module = slang
            .session
            .load_module_from_source_string(&name, &name, &source)
            .unwrap();

        let entry = module
            .find_entry_point_by_name("main")
            .expect("Failed to find `main` entry point");

        let program = slang
            .session
            .create_composite_component_type(&[module.downcast().clone(), entry.downcast().clone()])
            .expect("Failed to create program");

        let reflection = program
            .get_layout()
            .expect("Failed to get shader reflection");

        Self {
            path,
            reflection,
            _module: module,
            _program: program,
            _phantom: PhantomData::default(),
        }
    }

    pub fn from_path<P: Into<PathBuf>>(slang: &'a Slang, file_path: P) -> ShaderReflection<'a> {
        let shader_path = file_path.into();

        let module = slang
            .session
            .load_module(&shader_path.to_string_lossy())
            .unwrap();

        let entry = module
            .find_entry_point_by_name("main")
            .expect("Failed to find `main` entry point");

        let program = slang
            .session
            .create_composite_component_type(&[module.downcast().clone(), entry.downcast().clone()])
            .expect("Failed to create program");

        let reflection = program
            .get_layout()
            .expect("Failed to get shader reflection");

        Self {
            path: shader_path,
            reflection,
            _module: module,
            _program: program,
            _phantom: PhantomData::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let slang = Slang::new();
        let reflection = slang.from_path("shaders/simple.vert.slang");
        assert_eq!(reflection.get_entry_point_count(), 1);
    }
}
