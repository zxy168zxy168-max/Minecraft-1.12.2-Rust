use std::{collections::HashMap, io, path::Path, sync::Arc};

use crate::net::optifine::shader::config::ShaderPackOptions::ShaderPackOptions;
use crate::net::optifine::shader::IShaderPack::IShaderPack;

const MAX_INCLUDE_DEPTH: usize = 10;

/// Shared include-expansion cache used only while discovering shader options.
///
/// OptiFine 1.12.2 calls `ShaderPackParser.loadFile` for every program. Real
/// packs commonly include the same large option libraries from dozens of
/// programs, so resolving those libraries repeatedly dominates initial load.
/// The option parser only needs the original source lines and include content;
/// the compiler-only `MC_*` macro block and `#line` directives cannot define a
/// user option and are therefore intentionally omitted here. Cache keys retain
/// include depth so the original ten-level include limit is not bypassed.
#[derive(Debug, Default)]
pub struct ShaderOptionSourceCache {
    expanded: HashMap<(String, usize), Option<Arc<str>>>,
    resourceReads: u64,
    expansions: u64,
    cacheHits: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderOptionSourceStats {
    pub resourceReads: u64,
    pub expansions: u64,
    pub cacheHits: u64,
    pub residentEntries: usize,
}

impl ShaderOptionSourceCache {
    pub fn stats(&self) -> ShaderOptionSourceStats {
        ShaderOptionSourceStats {
            resourceReads: self.resourceReads,
            expansions: self.expansions,
            cacheHits: self.cacheHits,
            residentEntries: self.expanded.len(),
        }
    }
}

/// Option-discovery equivalent of OptiFine 1.12.2
/// `ShaderPackParser.loadFile/resolveIncludes` with shared resolved-include
/// reuse across top-level programs.
pub fn loadShaderOptionSource(
    pack: &mut dyn IShaderPack,
    path: &str,
    cache: &mut ShaderOptionSourceCache,
) -> io::Result<Option<Arc<str>>> {
    load_option_file(pack, normalize_pack_path(path), 0, cache)
}

fn load_option_file(
    pack: &mut dyn IShaderPack,
    path: String,
    includeDepth: usize,
    cache: &mut ShaderOptionSourceCache,
) -> io::Result<Option<Arc<str>>> {
    if includeDepth >= MAX_INCLUDE_DEPTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("#include depth exceeded: {includeDepth}, file: {path}"),
        ));
    }

    let key = (path.clone(), includeDepth);
    if let Some(cached) = cache.expanded.get(&key).cloned() {
        cache.cacheHits = cache.cacheHits.saturating_add(1);
        return Ok(cached);
    }

    cache.resourceReads = cache.resourceReads.saturating_add(1);
    let Some(bytes) = pack.getResourceAsStream(&path)? else {
        cache.expanded.insert(key, None);
        return Ok(None);
    };
    let text = String::from_utf8_lossy(&bytes);
    let directory = Path::new(path.trim_start_matches('/'))
        .parent()
        .map(|path| format!("/{}", path.to_string_lossy().replace('\\', "/")))
        .unwrap_or_else(|| "/shaders".to_owned());
    let mut output = String::with_capacity(text.len() + 128);

    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(include) = parse_include(trimmed) {
            let includePath = if include.starts_with('/') {
                normalize_pack_path(&format!("/shaders{include}"))
            } else {
                normalize_pack_path(&format!("{directory}/{include}"))
            };
            let included = load_option_file(pack, includePath.clone(), includeDepth + 1, cache)?
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Included file not found: {includePath}"),
                    )
                })?;
            output.push_str(included.trim_end_matches('\n'));
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    cache.expansions = cache.expansions.saturating_add(1);
    let output: Arc<str> = Arc::from(output);
    cache.expanded.insert(key, Some(Arc::clone(&output)));
    Ok(Some(output))
}

#[derive(Debug, Clone)]
pub struct ShaderMacroEnvironment {
    pub minecraftVersion: i32,
    pub glVersion: i32,
    pub glslVersion: i32,
    pub osMacro: &'static str,
    pub vendorMacro: &'static str,
    pub rendererMacro: &'static str,
    pub fxaaLevel: i32,
    pub normalMap: bool,
    pub specularMap: bool,
    pub renderQuality: f32,
    pub shadowQuality: f32,
    pub handDepth: f32,
    pub oldHandLight: bool,
    pub oldLighting: bool,
    /// OpenGL extension macros exposed by OptiFine as `MC_GL_*`.
    pub extensionMacros: Vec<String>,
}

impl Default for ShaderMacroEnvironment {
    fn default() -> Self {
        Self {
            minecraftVersion: 11202,
            glVersion: 330,
            glslVersion: 330,
            osMacro: if cfg!(target_os = "windows") {
                "MC_OS_WINDOWS"
            } else if cfg!(target_os = "macos") {
                "MC_OS_MAC"
            } else if cfg!(target_os = "linux") {
                "MC_OS_LINUX"
            } else {
                "MC_OS_OTHER"
            },
            vendorMacro: "MC_GL_VENDOR_OTHER",
            rendererMacro: "MC_GL_RENDERER_OTHER",
            fxaaLevel: 0,
            normalMap: true,
            specularMap: true,
            renderQuality: 1.0,
            shadowQuality: 1.0,
            handDepth: 0.125,
            oldHandLight: true,
            oldLighting: true,
            extensionMacros: Vec::new(),
        }
    }
}

impl ShaderMacroEnvironment {
    pub fn macroLines(&self) -> String {
        let mut lines = String::new();
        macro_int(&mut lines, "MC_VERSION", self.minecraftVersion);
        macro_int(&mut lines, "MC_GL_VERSION", self.glVersion);
        macro_int(&mut lines, "MC_GLSL_VERSION", self.glslVersion);
        macro_flag(&mut lines, self.osMacro);
        macro_flag(&mut lines, self.vendorMacro);
        macro_flag(&mut lines, self.rendererMacro);
        if self.fxaaLevel > 0 {
            macro_int(&mut lines, "MC_FXAA_LEVEL", self.fxaaLevel);
        }
        if self.normalMap {
            macro_flag(&mut lines, "MC_NORMAL_MAP");
        }
        if self.specularMap {
            macro_flag(&mut lines, "MC_SPECULAR_MAP");
        }
        macro_float(&mut lines, "MC_RENDER_QUALITY", self.renderQuality);
        macro_float(&mut lines, "MC_SHADOW_QUALITY", self.shadowQuality);
        macro_float(&mut lines, "MC_HAND_DEPTH", self.handDepth);
        if self.oldHandLight {
            macro_flag(&mut lines, "MC_OLD_HAND_LIGHT");
        }
        if self.oldLighting {
            macro_flag(&mut lines, "MC_OLD_LIGHTING");
        }
        lines
    }
}

fn macro_int(output: &mut String, name: &str, value: i32) {
    output.push_str("#define ");
    output.push_str(name);
    output.push(' ');
    output.push_str(&value.to_string());
    output.push('\n');
}

fn macro_float(output: &mut String, name: &str, value: f32) {
    output.push_str("#define ");
    output.push_str(name);
    output.push(' ');
    output.push_str(&format!("{value:.6}"));
    output.push('\n');
}

fn macro_flag(output: &mut String, name: &str) {
    output.push_str("#define ");
    output.push_str(name);
    output.push('\n');
}

/// OptiFine 1.12.2 `ShaderPackParser.loadFile` / `resolveIncludes` equivalent.
///
/// The selected pack owns all bytes. Includes are limited to ten levels, root
/// includes are resolved beneath `/shaders`, relative includes are resolved
/// beside the including source, and `#line` indices retain compiler diagnostics.
pub fn loadShaderSource(
    pack: &mut dyn IShaderPack,
    path: &str,
    macros: &ShaderMacroEnvironment,
) -> io::Result<Option<String>> {
    loadShaderSourceWithOptions(pack, path, macros, None)
}

pub fn loadShaderSourceWithOptions(
    pack: &mut dyn IShaderPack,
    path: &str,
    macros: &ShaderMacroEnvironment,
    options: Option<&ShaderPackOptions>,
) -> io::Result<Option<String>> {
    let mut files = Vec::new();
    let mut indices = HashMap::new();
    let source = load_file(
        pack,
        normalize_pack_path(path),
        0,
        0,
        &mut files,
        &mut indices,
        macros,
        options,
    )?;
    Ok(source)
}

fn load_file(
    pack: &mut dyn IShaderPack,
    path: String,
    fileIndex: usize,
    includeDepth: usize,
    files: &mut Vec<String>,
    indices: &mut HashMap<String, usize>,
    macros: &ShaderMacroEnvironment,
    options: Option<&ShaderPackOptions>,
) -> io::Result<Option<String>> {
    if includeDepth >= MAX_INCLUDE_DEPTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("#include depth exceeded: {includeDepth}, file: {path}"),
        ));
    }
    let Some(bytes) = pack.getResourceAsStream(&path)? else {
        return Ok(None);
    };
    let text = String::from_utf8_lossy(&bytes);
    let directory = Path::new(path.trim_start_matches('/'))
        .parent()
        .map(|path| format!("/{}", path.to_string_lossy().replace('\\', "/")))
        .unwrap_or_else(|| "/shaders".to_owned());
    let mut output = String::with_capacity(text.len() + 256);
    let mut sawVersion = false;
    let mut extensionInsertion = None;

    for (lineNumber, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !sawVersion && trimmed.starts_with("#version") {
            sawVersion = true;
            output.push_str(line);
            output.push('\n');
            output.push_str(&macros.macroLines());
            extensionInsertion = Some(output.len());
            output.push_str(&format!("#line {} {}\n", lineNumber + 2, fileIndex));
            continue;
        }

        if let Some(include) = parse_include(trimmed) {
            let includePath = if include.starts_with('/') {
                normalize_pack_path(&format!("/shaders{include}"))
            } else {
                normalize_pack_path(&format!("{directory}/{include}"))
            };
            let includeIndex = if let Some(index) = indices.get(&includePath).copied() {
                index
            } else {
                files.push(includePath.clone());
                let index = files.len();
                indices.insert(includePath.clone(), index);
                index
            };
            let included = load_file(
                pack,
                includePath.clone(),
                includeIndex,
                includeDepth + 1,
                files,
                indices,
                macros,
                options,
            )?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Included file not found: {includePath}"),
                )
            })?;
            output.push_str(&format!("#line 1 {includeIndex}\n"));
            output.push_str(included.trim_end_matches('\n'));
            output.push('\n');
            output.push_str(&format!("#line {} {}\n", lineNumber + 2, fileIndex));
            continue;
        }

        if let Some(options) = options {
            output.push_str(&options.applyLine(line));
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }

    if let Some(position) = extensionInsertion {
        let mut extensionLines = String::new();
        for extension in &macros.extensionMacros {
            if output.contains(extension) {
                extensionLines.push_str("#define ");
                extensionLines.push_str(extension);
                extensionLines.push('\n');
            }
        }
        if !extensionLines.is_empty() {
            output.insert_str(position, &extensionLines);
        }
    }

    // OptiFine only injects its macro block after the first #version it sees.
    // Included library files normally have no #version directive, so rejecting
    // them here would make real shader packs fail before the GL compiler. Do
    // not invent a version either: preserve the source when no directive is
    // present, matching ShaderPackParser.resolveIncludes/loadFile.
    Ok(Some(output))
}

// Do not rewrite or deduplicate pack macros here. OptiFine 1.12.2 only
// expands includes and injects its MC_* macro block. Rewriting top-level
// definitions changes preprocessor branch structure and can unbalance #if/#endif.

fn parse_include(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("#include")?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    let path = &rest[..end];
    if path.is_empty()
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'/' | b'.' | b'-'))
    {
        return None;
    }
    Some(path)
}

fn normalize_pack_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let mut components = Vec::new();
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            value => components.push(value),
        }
    }
    format!("/{}", components.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MemoryPack {
        name: String,
        resources: HashMap<String, Vec<u8>>,
    }

    impl IShaderPack for MemoryPack {
        fn getName(&self) -> &str {
            &self.name
        }
        fn getResourceAsStream(&mut self, name: &str) -> io::Result<Option<Vec<u8>>> {
            Ok(self.resources.get(name).cloned())
        }
        fn hasDirectory(&mut self, _name: &str) -> bool {
            false
        }
        fn close(&mut self) {}
    }

    #[test]
    fn expands_relative_and_root_includes_after_version_macros() {
        let mut pack = MemoryPack {
            name: "test".to_owned(),
            resources: HashMap::from([
                (
                    "/shaders/gbuffers_terrain.vsh".to_owned(),
                    b"#version 120\n#include \"lib/common.glsl\"\n#include \"/world.glsl\"\nvoid main(){}\n".to_vec(),
                ),
                ("/shaders/lib/common.glsl".to_owned(), b"const int A = 1;\n".to_vec()),
                ("/shaders/world.glsl".to_owned(), b"const int B = 2;\n".to_vec()),
            ]),
        };
        let source = loadShaderSource(
            &mut pack,
            "/shaders/gbuffers_terrain.vsh",
            &ShaderMacroEnvironment::default(),
        )
        .unwrap()
        .unwrap();
        assert!(source.starts_with("#version 120\n#define MC_VERSION 11202\n"));
        assert!(source.contains("const int A = 1;"));
        assert!(source.contains("const int B = 2;"));
        assert!(source.contains("#line 1 1"));
        assert!(source.contains("#line 1 2"));
    }

    #[test]
    fn accepts_versionless_include_library_like_optifine() {
        let mut pack = MemoryPack {
            name: "test".to_owned(),
            resources: HashMap::from([
                (
                    "/shaders/composite.vsh".to_owned(),
                    b"#version 120\n#include \"lib/common.glsl\"\nvoid main(){}\n".to_vec(),
                ),
                (
                    "/shaders/lib/common.glsl".to_owned(),
                    b"const float INCLUDED_VALUE = 1.0;\n".to_vec(),
                ),
            ]),
        };
        let source = loadShaderSource(
            &mut pack,
            "/shaders/composite.vsh",
            &ShaderMacroEnvironment::default(),
        )
        .unwrap()
        .unwrap();
        assert!(source.contains("const float INCLUDED_VALUE = 1.0;"));
        assert!(source.contains("#define MC_VERSION 11202"));
    }

    #[test]
    fn rejects_include_recursion_beyond_optifine_limit() {
        let mut resources = HashMap::new();
        resources.insert(
            "/shaders/root.vsh".to_owned(),
            b"#version 120\n#include \"0.glsl\"\n".to_vec(),
        );
        for index in 0..11 {
            resources.insert(
                format!("/shaders/{index}.glsl"),
                format!("#include \"{}.glsl\"\n", index + 1).into_bytes(),
            );
        }
        let mut pack = MemoryPack {
            name: "test".to_owned(),
            resources,
        };
        let error = loadShaderSource(
            &mut pack,
            "/shaders/root.vsh",
            &ShaderMacroEnvironment::default(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn emits_all_optifine_1122_global_shader_macros() {
        let environment = ShaderMacroEnvironment {
            fxaaLevel: 4,
            normalMap: true,
            specularMap: true,
            renderQuality: 1.414_213_5,
            shadowQuality: 0.5,
            handDepth: 0.125,
            oldHandLight: true,
            oldLighting: false,
            ..ShaderMacroEnvironment::default()
        };
        let lines = environment.macroLines();
        assert!(lines.contains("#define MC_FXAA_LEVEL 4\n"));
        assert!(lines.contains("#define MC_NORMAL_MAP\n"));
        assert!(lines.contains("#define MC_SPECULAR_MAP\n"));
        assert!(lines.contains("#define MC_RENDER_QUALITY 1.414214\n"));
        assert!(lines.contains("#define MC_SHADOW_QUALITY 0.500000\n"));
        assert!(lines.contains("#define MC_HAND_DEPTH 0.125000\n"));
        assert!(lines.contains("#define MC_OLD_HAND_LIGHT\n"));
        assert!(!lines.contains("#define MC_OLD_LIGHTING\n"));
    }

    #[test]
    fn injects_only_referenced_supported_extension_macros_like_optifine() {
        let mut pack = MemoryPack {
            name: "test".to_owned(),
            resources: HashMap::from([(
                "/shaders/composite.fsh".to_owned(),
                b"#version 120\n#ifdef MC_GL_EXT_gpu_shader4\n#endif\n".to_vec(),
            )]),
        };
        let mut environment = ShaderMacroEnvironment::default();
        environment.extensionMacros = vec![
            "MC_GL_EXT_gpu_shader4".to_owned(),
            "MC_GL_ARB_shader_texture_lod".to_owned(),
        ];
        let source = loadShaderSource(&mut pack, "/shaders/composite.fsh", &environment)
            .unwrap()
            .unwrap();
        assert!(source.contains("#define MC_GL_EXT_gpu_shader4\n"));
        assert!(!source.contains("#define MC_GL_ARB_shader_texture_lod\n"));
    }

    #[test]
    fn option_source_cache_reuses_shared_include_at_same_depth() {
        struct CountingPack {
            name: String,
            resources: HashMap<String, Vec<u8>>,
            reads: HashMap<String, usize>,
        }
        impl IShaderPack for CountingPack {
            fn getName(&self) -> &str {
                &self.name
            }
            fn getResourceAsStream(&mut self, name: &str) -> io::Result<Option<Vec<u8>>> {
                *self.reads.entry(name.to_owned()).or_default() += 1;
                Ok(self.resources.get(name).cloned())
            }
            fn hasDirectory(&mut self, _name: &str) -> bool {
                false
            }
            fn close(&mut self) {}
        }

        let mut pack = CountingPack {
            name: "test".to_owned(),
            resources: HashMap::from([
                (
                    "/shaders/a.vsh".to_owned(),
                    b"#version 120\n#include \"lib/options.glsl\"\n".to_vec(),
                ),
                (
                    "/shaders/b.vsh".to_owned(),
                    b"#version 120\n#include \"lib/options.glsl\"\n".to_vec(),
                ),
                (
                    "/shaders/lib/options.glsl".to_owned(),
                    b"#define TEST_OPTION // [0 1]\n".to_vec(),
                ),
            ]),
            reads: HashMap::new(),
        };
        let mut cache = ShaderOptionSourceCache::default();
        let first = loadShaderOptionSource(&mut pack, "/shaders/a.vsh", &mut cache)
            .unwrap()
            .unwrap();
        let second = loadShaderOptionSource(&mut pack, "/shaders/b.vsh", &mut cache)
            .unwrap()
            .unwrap();
        assert!(first.contains("#define TEST_OPTION"));
        assert!(second.contains("#define TEST_OPTION"));
        assert_eq!(pack.reads.get("/shaders/lib/options.glsl"), Some(&1));
        assert_eq!(cache.stats().cacheHits, 1);
    }

    #[test]
    fn option_source_cache_preserves_relative_root_and_depth_semantics() {
        let mut resources = HashMap::from([
            (
                "/shaders/root.vsh".to_owned(),
                b"#version 120\n#include \"lib/common.glsl\"\n#include \"/global.glsl\"\n".to_vec(),
            ),
            (
                "/shaders/lib/common.glsl".to_owned(),
                b"const int RELATIVE_OPTION = 1;\n".to_vec(),
            ),
            (
                "/shaders/global.glsl".to_owned(),
                b"const int ROOT_OPTION = 2;\n".to_vec(),
            ),
        ]);
        let mut pack = MemoryPack {
            name: "test".to_owned(),
            resources: resources.clone(),
        };
        let mut cache = ShaderOptionSourceCache::default();
        let source = loadShaderOptionSource(&mut pack, "/shaders/root.vsh", &mut cache)
            .unwrap()
            .unwrap();
        assert!(source.contains("const int RELATIVE_OPTION = 1;"));
        assert!(source.contains("const int ROOT_OPTION = 2;"));
        assert!(!source.contains("#define MC_VERSION"));
        assert!(!source.contains("#line"));

        resources.insert(
            "/shaders/deep.vsh".to_owned(),
            b"#include \"0.glsl\"\n".to_vec(),
        );
        for index in 0..11 {
            resources.insert(
                format!("/shaders/{index}.glsl"),
                format!("#include \"{}.glsl\"\n", index + 1).into_bytes(),
            );
        }
        let mut deepPack = MemoryPack {
            name: "test".to_owned(),
            resources,
        };
        let mut deepCache = ShaderOptionSourceCache::default();
        let error =
            loadShaderOptionSource(&mut deepPack, "/shaders/deep.vsh", &mut deepCache).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
