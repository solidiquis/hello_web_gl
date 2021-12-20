mod shaders;
use std::collections::HashMap;
use std::f32::consts::PI;
use js_sys::{Float32Array, Uint16Array};
use nalgebra_glm as glm;
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlCanvasElement, console};
use web_sys::WebGlBuffer as Buffer;
use web_sys::WebGlProgram as Program;
use web_sys::WebGlShader as Shader;
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlUniformLocation as UniformLocation;

#[wasm_bindgen]
pub fn hello_webgl() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id("hello-webgl").unwrap();

    let canvas = element.dyn_into::<HtmlCanvasElement>()?;
    let gl_context = canvas.get_context("webgl")?.unwrap().dyn_into::<GL>()?; 

    gl_context.enable(GL::BLEND | GL::DEPTH_TEST);
    gl_context.depth_func(GL::LEQUAL);
    gl_context.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
    gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
    gl_context.clear_depth(1.0);
    gl_context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

    console::log_1(&JsValue::from_str("Compiling shader_program..."));
    let shader_program = match ShaderProgram::new(&gl_context) {
        Ok(p) => {
            console::log_1(&JsValue::from_str("Shader program successfully compiled..."));
            p
        },
        Err(e) => {
            window.alert_with_message(&e)?;
            return Err(JsValue::from(1))
        }
    };

    let vert_index = *shader_program.attrloc.get("vertices").unwrap();
    let col_index = *shader_program.attrloc.get("colors").unwrap();
    let m_index = shader_program.unifloc.get("model_matrix").unwrap();
    let v_index = shader_program.unifloc.get("view_matrix").unwrap();
    let p_index = shader_program.unifloc.get("projection_matrix").unwrap();

    let (position_buffer, color_buffer, index_buffer) = ShaderProgram::init_vbo(&gl_context);

    let mut num_components = 3;
    let mut kind = GL::FLOAT;
    let normalize = false;
    let stride = 0;
    let offset = 0;

    gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));
    gl_context.vertex_attrib_pointer_with_i32(
        vert_index,
        num_components,
        kind,
        normalize,
        stride,
        offset
    );
    gl_context.enable_vertex_attrib_array(vert_index);

    gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    num_components = 4;

    gl_context.vertex_attrib_pointer_with_i32(
        col_index,
        num_components,
        kind,
        normalize,
        stride,
        offset
    );
    gl_context.enable_vertex_attrib_array(col_index);

    gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

    gl_context.use_program(Some(&shader_program.program));

    let inner_height = window.inner_height()?.as_f64().unwrap();
    let inner_width = window.inner_width()?.as_f64().unwrap();
    let aspect_ratio = (inner_width / inner_height) as f32;

    let model_matrix: &[f32] = &ShaderProgram::model_matrix();
    let view_matrix: &[f32]= &ShaderProgram::view_matrix();
    let projection_matrix: &[f32] = &ShaderProgram::projection_matrix(aspect_ratio);

    gl_context.uniform_matrix4fv_with_f32_array(
        Some(&m_index),
        false,
        model_matrix
    );
    gl_context.uniform_matrix4fv_with_f32_array(
        Some(&v_index),
        false,
        view_matrix
    );
    gl_context.uniform_matrix4fv_with_f32_array(
        Some(&p_index),
        false,
        projection_matrix
    );

    let vertex_count = 36;
    kind = GL::UNSIGNED_SHORT;
    gl_context.draw_elements_with_i32(GL::TRIANGLES, vertex_count, kind, offset);

    Ok(())
}

struct ShaderProgram {
    program: Program,
    attrloc: HashMap<String, u32>,
    unifloc: HashMap<String, UniformLocation>
}

impl ShaderProgram {
    fn new(gl_context: &GL) -> Result<Self, String> {
        console::log_1(&JsValue::from_str("Compiling vertex shader..."));
        let vert_shader = match Self::compile_shader(gl_context, GL::VERTEX_SHADER, shaders::VS_GLSL) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        console::log_1(&JsValue::from_str("Compiling fragment shader..."));
        let frag_shader = match Self::compile_shader(gl_context, GL::FRAGMENT_SHADER, shaders::FS_GLSL) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        let program = gl_context.create_program().unwrap();
        gl_context.attach_shader(&program, &vert_shader);
        gl_context.attach_shader(&program, &frag_shader);
        gl_context.link_program(&program);

        if let false = gl_context.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap() {
            let log = gl_context.get_program_info_log(&program).unwrap();
            let err = format!("An error occurred compiling shader program: {}", log);
            return Err(err)
        }

        let mut attribute_locations: HashMap<String, u32> = HashMap::new();
        attribute_locations.insert(
            "vertices".to_string(),
            gl_context.get_attrib_location(&program, "vCoord") as u32
        );
        attribute_locations.insert(
            "colors".to_string(),
            gl_context.get_attrib_location(&program, "vColor") as u32
        );

        let mut uniform_locations: HashMap<String, UniformLocation> = HashMap::new();
        uniform_locations.insert(
            "model_matrix".to_string(),
            gl_context.get_uniform_location(&program, "m").unwrap()
        );
        uniform_locations.insert(
            "view_matrix".to_string(),
            gl_context.get_uniform_location(&program, "v").unwrap()
        );
        uniform_locations.insert(
            "projection_matrix".to_string(),
            gl_context.get_uniform_location(&program, "p").unwrap()
        );

        Ok(Self {
            program: program,
            attrloc: attribute_locations,
            unifloc: uniform_locations
        })
    }

    fn compile_shader(gl_context: &GL, kind: u32, source: &'static str) -> Result<Shader, String> {
        let shader = gl_context.create_shader(kind).unwrap();
        gl_context.shader_source(&shader, source);
        gl_context.compile_shader(&shader);

        if let false = gl_context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap() {
            let log = gl_context.get_shader_info_log(&shader).unwrap();
            let err = format!("An error occurred compiling shader: {}", log);
            gl_context.delete_shader(Some(&shader));
            return Err(err)
        }

        Ok(shader) 
    }

    fn init_vbo(gl_context: &GL) -> (Buffer, Buffer, Buffer) {
        let position_buffer = gl_context.create_buffer().unwrap();
        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));

        let positions: [f32; 72] = [
            // Front face
            -1.0, -1.0,  1.0,
            1.0, -1.0,  1.0,
            1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,

            // Back face
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            1.0,  1.0, -1.0,
            1.0, -1.0, -1.0,

            // Top face
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            1.0,  1.0,  1.0,
            1.0,  1.0, -1.0,

            // Bottom face
            -1.0, -1.0, -1.0,
            1.0, -1.0, -1.0,
            1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,

            // Right face
            1.0, -1.0, -1.0,
            1.0,  1.0, -1.0,
            1.0,  1.0,  1.0,
            1.0, -1.0,  1.0,

            // Left face
            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,
        ];
        let sl_positions: &[f32] = &positions;
        let positions_ar = Float32Array::from(sl_positions).buffer();

        gl_context.buffer_data_with_opt_array_buffer(
            GL::ARRAY_BUFFER,
            Some(&positions_ar),
            GL::STATIC_DRAW
        );

        let index_buffer = gl_context.create_buffer().unwrap();
        gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        let indices: [u16; 36] = [
            0,  1,  2,      0,  2,  3,    // front
            4,  5,  6,      4,  6,  7,    // back
            8,  9,  10,     8,  10, 11,   // top
            12, 13, 14,     12, 14, 15,   // bottom
            16, 17, 18,     16, 18, 19,   // right
            20, 21, 22,     20, 22, 23,   // left
        ];
        let sl_indices: &[u16] = &indices;
        let indices_ar = Uint16Array::from(sl_indices).buffer();

        gl_context.buffer_data_with_opt_array_buffer(
            GL::ELEMENT_ARRAY_BUFFER,
            Some(&indices_ar),
            GL::STATIC_DRAW
        );

        let color_buffer = gl_context.create_buffer().unwrap();
        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));

        let colors: [f32; 96] = [
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0,
            0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0,
            0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
            1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0,
        ];

        let sl_colors: &[f32] = &colors;
        let colors_ar = Float32Array::from(sl_colors).buffer();

        gl_context.buffer_data_with_opt_array_buffer(
            GL::ARRAY_BUFFER,
            Some(&colors_ar),
            GL::STATIC_DRAW
        );

        (position_buffer, color_buffer, index_buffer)
    }

    fn model_matrix() -> Vec<f32> {
        let mut matrix = glm::rotate(
            &glm::TMat4::identity(),
            //PI / 4.0,
            0.0,
            &glm::vec3(0.0, 0.0, -1.0)
        );
        matrix = glm::translate(
            &matrix,
            &glm::vec3(0.0, 0.0, -5.0)
        );
        let model: [[f32; 4]; 4] = *matrix.as_ref();

        model.into_iter().flatten().collect::<Vec<f32>>()
    }

    fn view_matrix() -> Vec<f32> {
        let camera_position = glm::vec3(0.0, 0.0, 0.0);
        let camera_target = glm::vec3(0.0, 0.0, -5.0);
        let camera_up = glm::vec3(0.0, 1.0, 0.0);

        let matrix = glm::look_at(&camera_position, &camera_target, &camera_up);
        let view: [[f32; 4]; 4] = *matrix.as_ref();

        view.into_iter().flatten().collect::<Vec<f32>>()
    }

    fn projection_matrix(aspect_ratio: f32) -> Vec<f32> {
        let field_of_view = PI / 4.0;
        let near_plane_z_dist = 0.1;
        let far_plane_z_dist = 100.0;
        let matrix = glm::perspective(aspect_ratio, field_of_view, near_plane_z_dist, far_plane_z_dist);
        let projection: [[f32; 4]; 4] = *matrix.as_ref();

        projection.into_iter().flatten().collect::<Vec<f32>>()
    }
}

