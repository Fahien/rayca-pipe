# Rayca-Pipe

Rayca-Pipe is a Rust procedural macro library for parsing and reflecting Slang shader pipelines. It leverages the [Slang](https://github.com/shader-slang/slang) shading language and its reflection API to analyze shader modules and generate Rust code for graphics pipelines.

## Features

- **Procedural Macro**: The `pipewriter!` macro parses a Slang shader file at compile time and generates Rust code for pipeline structures.
- **Shader Reflection**: Uses Slang's reflection API to inspect shader entry points and stages.
- **Extensible Model**: The internal model supports multiple shader stages and can be extended for more advanced pipeline generation.

## Example

```rust
pipewriter!("path/to/shader.slang");
```

This macro will parse the specified Slang shader file and generate Rayca `Pipeline` source code based on its entry points.

## Project Structure

- `lib.rs`: Main entry point, defines the procedural macro and code generation logic.
- `model.rs`: Contains the data structures for representing pipelines and shaders.
- `parse.rs`: Handles parsing and reflection of Slang shader files.
