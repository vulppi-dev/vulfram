mod core;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::time::Instant;

use core::cmd::render::*;
use core::cmd::win::*;
use core::cmd::{EngineBatchCmds, EngineCmd, EngineCmdEnvelope};
use core::render::components::{Viewport, ViewportMode};
use core::render::enums::*;
use core::render::material_types::PrimitiveStateDesc;
use core::render::resources::*;

// MARK: - Vertex Structure

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

// MARK: - Shader Source

const TRIANGLE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

// MARK: - Host Simulation

/// Simula um host enviando comandos para desenhar um triângulo
fn main() {
    env_logger::init();

    println!("🦊 Vulfram Demo - Renderização de Triângulo\n");
    println!("   Simulação de host via MessagePack\n");

    // MARK: Initialize
    println!("📦 Inicializando engine...");
    let init_result = core::vulfram_init();
    if init_result != core::result::VulframResult::Success {
        eprintln!("❌ Falha ao inicializar: {:?}", init_result);
        std::process::exit(1);
    }
    println!("✅ Engine inicializada\n");

    // Handler Ctrl+C
    ctrlc::set_handler(move || {
        println!("\n\n👋 Sinal de interrupção...");
        std::process::exit(0);
    })
    .expect("Falha ao registrar handler Ctrl+C");

    // MARK: Setup Scene
    println!("🎨 Configurando cena...\n");

    let mut cmd_id = 1u64;
    let mut window_id = 0u32; // Will be set from response
    let shader_id = 1u32;
    let geometry_id = 1u32;
    let material_id = 1u32;
    let camera_id = 1u32;
    let model_id = 1u32;

    // Step 1: Create Window
    println!("1️⃣ Criando janela...");
    let mut batch = Vec::new();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
            title: "Vulfram Triangle Demo".into(),
            size: glam::UVec2::new(800, 600),
            resizable: true,
            ..Default::default()
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Tick para processar criação da janela
    core::vulfram_tick(0, 0);
    let (success, received_id) = read_window_create_response();
    if !success {
        eprintln!("❌ Falha ao criar janela!");
        return;
    }
    window_id = received_id;
    println!("   Window ID recebido: {}\n", window_id);
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 2: Upload Shader
    println!("2️⃣ Fazendo upload do shader...");
    let shader_bytes = TRIANGLE_SHADER.as_bytes();
    let buffer_id_shader = 1001u64;
    upload_buffer(buffer_id_shader, shader_bytes);

    // Step 3: Create Shader
    println!("3️⃣ Criando shader...");
    batch.clear();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdShaderCreate(CmdShaderCreateArgs {
            shader_id,
            window_id,
            buffer_id: buffer_id_shader,
            label: Some("Triangle Shader".into()),
            vertex_attributes: vec![
                VertexAttributeSpec {
                    location: 0,
                    semantic: VertexSemantic::Position,
                    format: VertexFormat::Float32x3,
                },
                VertexAttributeSpec {
                    location: 1,
                    semantic: VertexSemantic::Color0,
                    format: VertexFormat::Float32x3,
                },
            ],
            uniform_buffers: vec![],
            texture_bindings: vec![],
            storage_buffers: vec![],
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    core::vulfram_tick(100, 100);
    read_responses();

    // Step 4: Upload Geometry
    println!("4️⃣ Fazendo upload da geometria...");
    let vertices = [
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        }, // Top - Red
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0],
        }, // Bottom Left - Green
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        }, // Bottom Right - Blue
    ];
    let vertex_bytes = bytemuck::cast_slice(&vertices);
    let buffer_id_vertex = 1002u64;
    upload_buffer(buffer_id_vertex, vertex_bytes);

    let indices: [u16; 3] = [0, 1, 2];
    let index_bytes = bytemuck::cast_slice(&indices);
    let buffer_id_index = 1003u64;
    upload_buffer(buffer_id_index, index_bytes);

    // Step 5: Create Geometry
    println!("5️⃣ Criando geometria...");
    batch.clear();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdGeometryCreate(CmdGeometryCreateArgs {
            geometry_id,
            window_id,
            vertex_buffer_id: buffer_id_vertex,
            index_buffer_id: buffer_id_index,
            vertex_count: 3,
            index_count: 3,
            vertex_attributes: vec![
                VertexAttributeDesc {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttributeDesc {
                    format: VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
            ],
            index_format: IndexFormat::Uint16,
            label: Some("Triangle Geometry".into()),
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    core::vulfram_tick(200, 100);
    read_responses();

    // Step 6: Create Material
    println!("6️⃣ Criando material...");
    batch.clear();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdMaterialCreate(CmdMaterialCreateArgs {
            material_id,
            window_id,
            shader_id,
            textures: vec![],
            blend: None,
            depth_stencil: None,
            primitive: PrimitiveStateDesc {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(CullMode::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            label: Some("Triangle Material".into()),
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    core::vulfram_tick(300, 100);
    read_responses();

    // Step 7: Create Camera
    println!("7️⃣ Criando câmera...");
    let proj_mat = Mat4::IDENTITY;
    let view_mat = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 2.0), Vec3::ZERO, Vec3::Y);

    batch.clear();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdCameraCreate(CmdCameraCreateArgs {
            component_id: camera_id,
            window_id,
            proj_mat,
            view_mat,
            viewport: Viewport {
                position_mode: ViewportMode::Absolute,
                size_mode: ViewportMode::Absolute,
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                anchor: glam::Vec2::ZERO,
            },
            layer_mask: 0xFFFFFFFF,
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    core::vulfram_tick(400, 100);
    read_responses();

    // Step 8: Create Model
    println!("8️⃣ Criando modelo...");
    batch.clear();
    batch.push(EngineCmdEnvelope {
        id: cmd_id,
        cmd: EngineCmd::CmdModelCreate(CmdModelCreateArgs {
            component_id: model_id,
            window_id,
            geometry_id,
            material_id,
            model_mat: Mat4::IDENTITY,
            layer_mask: 0xFFFFFFFF,
        }),
    });
    cmd_id += 1;
    send_commands(&batch);
    core::vulfram_tick(500, 100);
    read_responses();

    println!("\n✅ Cena configurada!\n");

    // MARK: Main Loop
    println!("🎮 Loop de renderização...");
    println!("   Renderizando por 5 segundos\n");

    let start_time = Instant::now();
    let mut last_time = start_time;
    let mut running = true;
    let mut frame_count = 0u64;

    while running {
        let current_time = Instant::now();
        let elapsed_ms = current_time.duration_since(start_time).as_millis() as u64;
        let delta_ms = current_time.duration_since(last_time).as_millis() as u32;
        last_time = current_time;

        // Tick engine
        let tick_result = core::vulfram_tick(elapsed_ms, delta_ms);
        if tick_result != core::result::VulframResult::Success {
            eprintln!("❌ Erro no tick: {:?}", tick_result);
            break;
        }

        // Read events
        read_events();

        frame_count += 1;

        // Log a cada 60 frames (~1s)
        if frame_count % 60 == 0 {
            let fps = 1000.0 / delta_ms.max(1) as f32;
            println!("📊 Frame {}: {:.1} FPS", frame_count, fps);
        }

        // Sleep ~60 FPS
        std::thread::sleep(std::time::Duration::from_millis(16));

        // Roda por 5 segundos
        if elapsed_ms > 5000 {
            println!("\n✅ Demo concluída!");
            running = false;
        }
    }

    // MARK: Cleanup
    println!("\n🧹 Limpando recursos...");
    core::vulfram_dispose();
    println!("✅ Finalizado\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "✨ Triângulo renderizado - {} frames em {:.2}s",
        frame_count,
        frame_count as f32 / 60.0
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

// MARK: - Helper Functions

/// Free buffer allocated by core (for receive_queue and receive_events)
fn free_core_buffer(ptr: *const u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len) as *mut [u8]);
        }
    }
}

fn upload_buffer(buffer_id: u64, data: &[u8]) {
    let result = core::vulfram_upload_buffer(buffer_id, 0, data.as_ptr(), data.len());
    if result != core::result::VulframResult::Success {
        eprintln!("⚠️ Falha no upload do buffer {}: {:?}", buffer_id, result);
    }
}

fn send_commands(batch: &EngineBatchCmds) {
    match rmp_serde::to_vec(batch) {
        Ok(bytes) => {
            let result = core::vulfram_send_queue(bytes.as_ptr(), bytes.len());
            if result != core::result::VulframResult::Success {
                eprintln!("⚠️ Falha ao enviar comandos: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("⚠️ Falha ao serializar comandos: {}", e);
        }
    }
}

fn read_window_create_response() -> (bool, u32) {
    let mut ptr: *const u8 = std::ptr::null();
    let mut len: usize = 0;
    let mut window_id = 0u32;
    let mut success = false;

    let result = core::vulfram_receive_queue(&mut ptr, &mut len);
    if result == core::result::VulframResult::Success && !ptr.is_null() && len > 0 {
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            match rmp_serde::from_slice::<core::cmd::EngineBatchResponses>(slice) {
                Ok(responses) => {
                    for response in responses {
                        if let core::cmd::CommandResponse::WindowCreate(ref result) =
                            response.response
                        {
                            if result.success {
                                window_id = result.content;
                                success = true;
                                println!("   ← Janela criada com sucesso! ID: {}", window_id);
                            } else {
                                eprintln!("   ← Erro ao criar janela: {}", result.message);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Falha ao desserializar respostas: {}", e);
                }
            }
            free_core_buffer(ptr, len);
        }
    }
    (success, window_id)
}

fn read_responses() {
    let mut ptr: *const u8 = std::ptr::null();
    let mut len: usize = 0;

    let result = core::vulfram_receive_queue(&mut ptr, &mut len);
    if result == core::result::VulframResult::Success && !ptr.is_null() && len > 0 {
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            match rmp_serde::from_slice::<core::cmd::EngineBatchResponses>(slice) {
                Ok(responses) => {
                    for response in responses {
                        match &response.response {
                            core::cmd::CommandResponse::ShaderCreate(r) => {
                                if r.success {
                                    println!("   ← Shader criado com sucesso");
                                } else {
                                    eprintln!("   ← Erro: {}", r.message);
                                }
                            }
                            core::cmd::CommandResponse::GeometryCreate(r) => {
                                if r.success {
                                    println!("   ← Geometria criada com sucesso");
                                } else {
                                    eprintln!("   ← Erro: {}", r.message);
                                }
                            }
                            core::cmd::CommandResponse::MaterialCreate(r) => {
                                if r.success {
                                    println!("   ← Material criado com sucesso");
                                } else {
                                    eprintln!("   ← Erro: {}", r.message);
                                }
                            }
                            core::cmd::CommandResponse::CameraCreate(r) => {
                                if r.success {
                                    println!("   ← Câmera criada com sucesso");
                                } else {
                                    eprintln!("   ← Erro: {}", r.message);
                                }
                            }
                            core::cmd::CommandResponse::ModelCreate(r) => {
                                if r.success {
                                    println!("   ← Modelo criado com sucesso");
                                } else {
                                    eprintln!("   ← Erro: {}", r.message);
                                }
                            }
                            _ => {
                                println!("   ← Resposta: {:?}", response.response);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Falha ao desserializar respostas: {}", e);
                }
            }
            free_core_buffer(ptr, len);
        }
    }
}

fn read_events() {
    let mut ptr: *const u8 = std::ptr::null();
    let mut len: usize = 0;

    let result = core::vulfram_receive_events(&mut ptr, &mut len);
    if result == core::result::VulframResult::Success && !ptr.is_null() && len > 0 {
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            match rmp_serde::from_slice::<core::cmd::EngineBatchEvents>(slice) {
                Ok(events) => {
                    for event in events {
                        // Só loga eventos importantes
                        match &event {
                            core::cmd::EngineEvent::Window(w) => {
                                println!("   ← Window Event: {:?}", w);
                            }
                            core::cmd::EngineEvent::Keyboard(k) => {
                                println!("   ← Keyboard Event: {:?}", k);
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Falha ao desserializar eventos: {}", e);
                }
            }
            free_core_buffer(ptr, len);
        }
    }
}
