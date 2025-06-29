use crate::model::*;

use std::path::PathBuf;

pub fn parse_shader(file_path_str: &str) -> Pipeline {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shader_path = root_dir.join(file_path_str);
    let shader_parent = shader_path.parent().unwrap();

    let global_session = slang::GlobalSession::new().unwrap();

    let targets = [slang::TargetDesc::default()
        .format(slang::CompileTarget::Spirv)
        .profile(global_session.find_profile("sm_6_5"))];

    let shader_parent_str = shader_parent.to_string_lossy();
    let search_path = std::ffi::CString::new(shader_parent_str.as_bytes()).unwrap();
    let search_paths = [search_path.as_ptr()];

    // All compiler options are available through this builder.
    let session_options = slang::CompilerOptions::default()
        .optimization(slang::OptimizationLevel::High)
        .matrix_layout_row(true);

    let session_desc = slang::SessionDesc::default()
        .targets(&targets)
        .search_paths(&search_paths)
        .options(&session_options);

    let session = global_session.create_session(&session_desc).unwrap();

    let module = session.load_module(&shader_path.to_string_lossy()).unwrap();

    let entry_point = module.find_entry_point_by_name("main").unwrap();

    use slang::Downcast;
    let program = session
        .create_composite_component_type(&[
            module.downcast().clone(),
            entry_point.downcast().clone(),
        ])
        .expect("Failed to create program");

    let reflection = program
        .get_layout()
        .expect("Failed to get shader reflection");

    let mut pipeline = Pipeline::default();

    let entry_point_count = reflection.get_entry_point_count();
    for i in 0..entry_point_count {
        if let Some(entry_point) = reflection.get_entry_point_by_index(i) {
            match entry_point.get_stage() {
                slang::Stage::None => todo!(),
                slang::Stage::Vertex => pipeline.functions.push(Shader::default()),
                slang::Stage::Hull => todo!(),
                slang::Stage::Domain => todo!(),
                slang::Stage::Geometry => todo!(),
                slang::Stage::Fragment => todo!(),
                slang::Stage::Compute => todo!(),
                slang::Stage::RayGeneration => todo!(),
                slang::Stage::Intersection => todo!(),
                slang::Stage::AnyHit => todo!(),
                slang::Stage::ClosestHit => todo!(),
                slang::Stage::Miss => todo!(),
                slang::Stage::Callable => todo!(),
                slang::Stage::Mesh => todo!(),
                slang::Stage::Amplification => todo!(),
                slang::Stage::Count => todo!(),
            }
        }
    }

    pipeline
}
