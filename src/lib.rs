mod geometry;
mod maths;
mod shaders;
use std::f32::consts::PI;
use std::collections::HashMap;
use js_sys::{JsString, Float32Array, Number, Uint16Array};
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
struct HelloWebGL {
    gl_context: GL,
    program: Program,
    uniform_locations: HashMap<String, UniformLocation>,
    index_buffer: Buffer,
}

#[wasm_bindgen]
impl HelloWebGL {

    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: JsString) -> Self {
        // *======== Initialize context ========*
        let gl_context = match Self::init_context(canvas_id) {
            Ok(c) => c,
            Err(e) => wasm_bindgen::throw_val(e)
        };

        // *======== Load geometry into context ========*
        let vertex_buffer = if let Some(buf) = gl_context.create_buffer() {
            gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));        

            let slice: &[f32] = &geometry::VERTICES;
            let array = Float32Array::from(slice).buffer();
            gl_context.buffer_data_with_opt_array_buffer(
                GL::ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
            );

            buf
        } else {
            let err = JsValue::from_str("Error binding data to vertex buffer.");
            wasm_bindgen::throw_val(err);
        };

        let color_buffer = if let Some(buf) = gl_context.create_buffer() {
            gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));

            let slice: &[f32] = &geometry::COLORS;
            let array = Float32Array::from(slice).buffer();
            gl_context.buffer_data_with_opt_array_buffer(
                GL::ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
            );

            buf
        } else {
            let err = JsValue::from_str("Error binding data to color buffer.");
            wasm_bindgen::throw_val(err);
        };

        let index_buffer = if let Some(buf) = gl_context.create_buffer() {
            gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buf));

            let slice: &[u16] = &geometry::INDICES;
            let array = Uint16Array::from(slice).buffer();
            gl_context.buffer_data_with_opt_array_buffer(
                GL::ELEMENT_ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
            );

            buf
        } else {
            let err = JsValue::from_str("Error binding data to index buffer.");
            wasm_bindgen::throw_val(err);
        };

        // *======== Compile shaders =========*
        let vertex_shader = if let Some(shader) = gl_context.create_shader(GL::VERTEX_SHADER) {
            gl_context.shader_source(&shader, shaders::VS_GLSL);
            gl_context.compile_shader(&shader);

            if let false = gl_context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap() {
                let log = gl_context.get_shader_info_log(&shader).unwrap();
                let err = JsValue::from(format!("An error occurred compiling shader: {}", log));
                gl_context.delete_shader(Some(&shader));
                wasm_bindgen::throw_val(err);
            }

            shader
        } else {
            let err = JsValue::from_str("Failed to create vertex shader.");
            wasm_bindgen::throw_val(err);
        };

        let fragment_shader = if let Some(shader) = gl_context.create_shader(GL::FRAGMENT_SHADER) {
            gl_context.shader_source(&shader, shaders::FS_GLSL);
            gl_context.compile_shader(&shader);

            if let false = gl_context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap() {
                let log = gl_context.get_shader_info_log(&shader).unwrap();
                let err = JsValue::from(format!("An error occurred compiling shader: {}", log));
                gl_context.delete_shader(Some(&shader));
                wasm_bindgen::throw_val(err);
            }

            shader
        } else {
            let err = JsValue::from_str("Failed to create vertex shader.");
            wasm_bindgen::throw_val(err);
        };

        // *======== Initialize shader program ========*
        let program = match gl_context.create_program() {
            Some(p) => p,
            None => {
                let err = JsValue::from_str("Failed to initialize shader program");
                wasm_bindgen::throw_val(err)
            }
        };

        gl_context.attach_shader(&program, &vertex_shader);
        gl_context.attach_shader(&program, &fragment_shader);
        gl_context.link_program(&program);

        if let false = gl_context.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap() {
            let log = gl_context.get_program_info_log(&program).unwrap();
            let err = JsValue::from_str(&format!("An error occurred compiling shader program: {}", log));
            wasm_bindgen::throw_val(err);
        }

        // *======== Mapping attributes to shaders =========*
        let mut uniform_locations = HashMap::new();

        if let Some(m) = gl_context.get_uniform_location(&program, "m") {
            uniform_locations.insert("m".to_string(), m);
        } else {
            let err = JsValue::from_str("Failed to get uniform, m, location.");
            wasm_bindgen::throw_val(err);
        }

        if let Some(v) = gl_context.get_uniform_location(&program, "v") {
            uniform_locations.insert("v".to_string(), v);
        } else {
            let err = JsValue::from_str("Failed to get uniform, v, location.");
            wasm_bindgen::throw_val(err);
        }

        if let Some(p) = gl_context.get_uniform_location(&program, "p") {
            uniform_locations.insert("p".to_string(), p);
        } else {
            let err = JsValue::from_str("Failed to get uniform, p, location.");
            wasm_bindgen::throw_val(err);
        }

        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        let position = gl_context.get_attrib_location(&program, "position") as u32;
        gl_context.vertex_attrib_pointer_with_i32(position, 3, GL::FLOAT, false, 0, 0);
        gl_context.enable_vertex_attrib_array(position);

        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
        let color = gl_context.get_attrib_location(&program, "color") as u32;
        gl_context.vertex_attrib_pointer_with_i32(color, 3, GL::FLOAT, false, 0, 0);
        gl_context.enable_vertex_attrib_array(color);

        // *======== Attach shader program to context ========*
        gl_context.use_program(Some(&program));

        Self { gl_context, program, uniform_locations, index_buffer }
    }

    #[wasm_bindgen]
    pub fn render(&self, canvas_width: Number, canvas_height: Number) {
        let width = canvas_width.as_f64().unwrap();
        let height = canvas_height.as_f64().unwrap();

        self.gl_context.enable(GL::DEPTH_TEST);
        self.gl_context.depth_func(GL::LEQUAL);
        self.gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl_context.clear_depth(1.0);
        self.gl_context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let m_loc = self.uniform_locations.get("m").unwrap();
        let v_loc = self.uniform_locations.get("v").unwrap();
        let p_loc = self.uniform_locations.get("p").unwrap();

        self.gl_context.uniform_matrix4fv_with_f32_array(Some(m_loc), false, &maths::model_matrix());
        self.gl_context.uniform_matrix4fv_with_f32_array(Some(v_loc), false, &maths::view_matrix());
        self.gl_context.uniform_matrix4fv_with_f32_array(Some(p_loc), false, &maths::projection_matrix(width, height));

        self.gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        self.gl_context.draw_elements_with_i32(GL::TRIANGLES, 36, GL::UNSIGNED_SHORT, 0);
    }

    fn init_context(canvas_id: JsString) -> Result<GL, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element_id = String::from(canvas_id);

        let element = match document.get_element_by_id(&element_id) {
            Some(e) => e,
            None => {
                let err = JsValue::from_str(&format!(
                    "No canvas found with id: {}", element_id
                ));
                return Err(err)
            }
        };

        let canvas = element.dyn_into::<HtmlCanvasElement>()?;
        let gl_context = canvas.get_context("webgl")?.unwrap().dyn_into::<GL>()?; 

        Ok(gl_context)
    }
}
