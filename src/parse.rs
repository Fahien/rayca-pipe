// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use slang::Downcast;

pub struct ShaderReflection {
    pub path: PathBuf,
    reflection: slang::ShaderReflection,
    _program: slang::ComponentType,
    _module: slang::Module,
    _session: slang::Session,
    _global_session: slang::GlobalSession,
}

impl std::ops::Deref for ShaderReflection {
    type Target = slang::ShaderReflection;

    fn deref(&self) -> &Self::Target {
        &self.reflection
    }
}

impl ShaderReflection {
    pub fn parse(file_path_str: &str) -> ShaderReflection {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let shader_path = root_dir.join(file_path_str);
        let shader_parent = shader_path
            .parent()
            .expect("Failed to get parent from shader path");

        let global_session = slang::GlobalSession::new().unwrap();

        let shader_parent_str = shader_parent.to_string_lossy();
        let shader_parent_cstr = std::ffi::CString::new(shader_parent_str.as_bytes())
            .expect("Failed to create CString for shader search path");
        let search_paths = [shader_parent_cstr.as_ptr()];

        // All compiler options are available through this builder.
        let session_options = slang::CompilerOptions::default()
            .optimization(slang::OptimizationLevel::High)
            .matrix_layout_row(true);

        let targets = [slang::TargetDesc::default()
            .format(slang::CompileTarget::Spirv)
            .profile(global_session.find_profile("sm_6_5"))];

        let session_desc = slang::SessionDesc::default()
            .targets(&targets)
            .search_paths(&search_paths)
            .options(&session_options);

        let session = global_session.create_session(&session_desc).unwrap();

        let module = session.load_module(&shader_path.to_string_lossy()).unwrap();

        let entry = module
            .find_entry_point_by_name("main")
            .expect("Failed to find `main` entry point");

        let program = session
            .create_composite_component_type(&[module.downcast().clone(), entry.downcast().clone()])
            .expect("Failed to create program");

        let reflection = program
            .get_layout()
            .expect("Failed to get shader reflection");

        Self {
            path: PathBuf::from(file_path_str),
            reflection,
            _module: module,
            _session: session,
            _program: program,
            _global_session: global_session,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let reflection = ShaderReflection::parse("shaders/simple.vert.slang");
        assert_eq!(reflection.get_entry_point_count(), 1);
    }
}
