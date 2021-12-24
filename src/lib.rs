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
pub fn hello_webgl() {
    match demo() {
        Ok(_) => console::log_1(&JsValue::from_str("Demo successfully rendered!")),
        Err(e) => {
            console::error_1(&e);
            wasm_bindgen::throw_val(e);
        }
    }
}

fn demo() -> Result<(), JsValue> {
    // *======== Canvas & WebGL Context =========*
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let element = document.get_element_by_id("hello-webgl").unwrap();

    let canvas = element.dyn_into::<HtmlCanvasElement>()?;
    let gl_context = canvas.get_context("webgl")?.unwrap().dyn_into::<GL>()?; 

    // *======== Geometry =========*
    let vertices: [f32; 72] = [
        -1.0,-1.0,-1.0,   1.0,-1.0,-1.0,   1.0, 1.0,-1.0,  -1.0, 1.0,-1.0,
        -1.0,-1.0, 1.0,   1.0,-1.0, 1.0,   1.0, 1.0, 1.0,  -1.0, 1.0, 1.0,
        -1.0,-1.0,-1.0,  -1.0, 1.0,-1.0,  -1.0, 1.0, 1.0,  -1.0,-1.0, 1.0,
         1.0,-1.0,-1.0,   1.0, 1.0,-1.0,   1.0, 1.0, 1.0,   1.0,-1.0, 1.0,
        -1.0,-1.0,-1.0,  -1.0,-1.0, 1.0,   1.0,-1.0, 1.0,   1.0,-1.0,-1.0,
        -1.0, 1.0,-1.0,  -1.0, 1.0, 1.0,   1.0, 1.0, 1.0,   1.0, 1.0,-1.0
    ];

    let colors: [f32; 72] = [
        1.0,0.0,0.0,  1.0,0.0,0.0,  1.0,0.0,0.0,  1.0,0.0,0.0,
        0.0,1.0,0.0,  0.0,1.0,0.0,  0.0,1.0,0.0,  0.0,1.0,0.0,
        1.0,0.0,1.0,  0.0,0.0,1.0,  0.0,0.0,1.0,  0.0,0.0,1.0,
        1.0,1.0,0.0,  1.0,1.0,0.0,  1.0,1.0,0.0,  1.0,1.0,0.0,
        1.0,0.0,1.0,  1.0,0.0,1.0,  1.0,0.0,1.0,  1.0,0.0,1.0,
        0.0,1.0,1.0,  0.0,1.0,1.0,  0.0,1.0,1.0,  0.0,1.0,1.0,
    ];

    let indices: [u16; 36] = [
        0, 1, 2,   0, 2, 3,   4, 5, 6,   4,6,7,
        8, 9, 10,  8, 10,11,  12,13,14,  12,14,15,
        16,17,18,  16,18,19,  20,21,22,  20,22,23
    ];

    // *======== Load geometry into context =========*
    let vertex_buffer = if let Some(buf) = gl_context.create_buffer() {
        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));        

        let slice: &[f32] = &vertices;
        let array = Float32Array::from(slice).buffer();
        gl_context.buffer_data_with_opt_array_buffer(
            GL::ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
        );

        buf
    } else {
        let err = JsValue::from_str("Error binding data to vertex buffer.");
        return Err(err);
    };

    let color_buffer = if let Some(buf) = gl_context.create_buffer() {
        gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));

        let slice: &[f32] = &colors;
        let array = Float32Array::from(slice).buffer();
        gl_context.buffer_data_with_opt_array_buffer(
            GL::ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
        );

        buf
    } else {
        let err = JsValue::from_str("Error binding data to color buffer.");
        return Err(err);
    };

    let index_buffer = if let Some(buf) = gl_context.create_buffer() {
        gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buf));

        let slice: &[u16] = &indices;
        let array = Uint16Array::from(slice).buffer();
        gl_context.buffer_data_with_opt_array_buffer(
            GL::ELEMENT_ARRAY_BUFFER, Some(&array), GL::STATIC_DRAW
        );

        buf
    } else {
        let err = JsValue::from_str("Error indices data to index buffer.");
        return Err(err);
    };

    // *======== Compile shaders =========*
    let vertex_shader = if let Some(shader) = gl_context.create_shader(GL::VERTEX_SHADER) {
        gl_context.shader_source(&shader, shaders::VS_GLSL);
        gl_context.compile_shader(&shader);

        if let false = gl_context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap() {
            let log = gl_context.get_shader_info_log(&shader).unwrap();
            let err = JsValue::from(format!("An error occurred compiling shader: {}", log));
            gl_context.delete_shader(Some(&shader));
            return Err(err)
        }

        shader
    } else {
        let err = JsValue::from_str("Failed to create vertex shader.");
        return Err(err);
    };

    let fragment_shader = if let Some(shader) = gl_context.create_shader(GL::FRAGMENT_SHADER) {
        gl_context.shader_source(&shader, shaders::FS_GLSL);
        gl_context.compile_shader(&shader);

        if let false = gl_context.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap() {
            let log = gl_context.get_shader_info_log(&shader).unwrap();
            let err = JsValue::from(format!("An error occurred compiling shader: {}", log));
            gl_context.delete_shader(Some(&shader));
            return Err(err)
        }

        shader
    } else {
        let err = JsValue::from_str("Failed to create vertex shader.");
        return Err(err);
    };

    let program = if let Some(p) = gl_context.create_program() {
        p
    } else {
        let err = JsValue::from_str("Failed to initialize shader program");
        return Err(err);
    };

    gl_context.attach_shader(&program, &vertex_shader);
    gl_context.attach_shader(&program, &fragment_shader);
    gl_context.link_program(&program);

    if let false = gl_context.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap() {
        let log = gl_context.get_program_info_log(&program).unwrap();
        let err = JsValue::from_str(&format!("An error occurred compiling shader program: {}", log));
        return Err(err)
    }

    // *======== Mapping attributes to shaders =========*
    let m_mat = if let Some(m) = gl_context.get_uniform_location(&program, "m") {
        m
    } else {
        let err = JsValue::from_str("Failed to get uniform, m, location.");
        return Err(err)
    };

    let v_mat = if let Some(v) = gl_context.get_uniform_location(&program, "v") {
        v
    } else {
        let err = JsValue::from_str("Failed to get uniform, v, location.");
        return Err(err)
    };

    let p_mat = if let Some(p) = gl_context.get_uniform_location(&program, "p") {
        p
    } else {
        let err = JsValue::from_str("Failed to get uniform, p, location.");
        return Err(err)
    };
    
    gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
    let position = gl_context.get_attrib_location(&program, "position") as u32;
    gl_context.vertex_attrib_pointer_with_i32(position, 3, GL::FLOAT, false, 0, 0);
    gl_context.enable_vertex_attrib_array(position);

    gl_context.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
    let color = gl_context.get_attrib_location(&program, "color") as u32;
    gl_context.vertex_attrib_pointer_with_i32(color, 3, GL::FLOAT, false, 0, 0);
    gl_context.enable_vertex_attrib_array(color);

    gl_context.use_program(Some(&program));

    // *======== Maths =========*
    let model_matrix = (|| {
        let identity = glm::TMat4::identity();
        let rotate = glm::rotate(&identity, PI / 4.0, &glm::vec3(0.0, 1.0, 0.0));
        let transl = glm::translate(&identity, &glm::vec3(0.0, 0.0, -5.0));
        let mat = transl * rotate;

        let model: [[f32; 4]; 4] = *mat.as_ref();

        model.into_iter().flatten().collect::<Vec<f32>>()
    })();

    let view_matrix = (|| {
        let cam_position = glm::vec3(0.0, 0.0, 0.0);
        let cam_target = glm::vec3(0.0, 0.0, -1.0);
        let cam_up = glm::vec3(0.0, 1.0, 0.0);

        let mat = glm::look_at(&cam_position, &cam_target, &cam_up);
        let view: [[f32; 4]; 4] = *mat.as_ref();

        view.into_iter().flatten().collect::<Vec<f32>>()
    })();

    let projection_matrix = (|| {
        let aspect_ratio = canvas.width() as f32 / canvas.height() as f32;
        let fov = PI / 4.0;
        let near = 0.1;
        let far = 100.0;
        let mat = glm::perspective(aspect_ratio, fov, near, far);
        let projection: [[f32; 4]; 4] = *mat.as_ref();

        projection.into_iter().flatten().collect::<Vec<f32>>()
    })();

    // *======== Draw time =========*
    gl_context.enable(GL::DEPTH_TEST);
    gl_context.depth_func(GL::LEQUAL);
    gl_context.clear_color(0.0, 0.0, 0.0, 1.0);
    gl_context.clear_depth(1.0);
    gl_context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

    gl_context.uniform_matrix4fv_with_f32_array(Some(&m_mat), false, &model_matrix);
    gl_context.uniform_matrix4fv_with_f32_array(Some(&v_mat), false, &view_matrix);
    gl_context.uniform_matrix4fv_with_f32_array(Some(&p_mat), false, &projection_matrix);

    gl_context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl_context.draw_elements_with_i32(GL::TRIANGLES, 36, GL::UNSIGNED_SHORT, 0);

    Ok(())
}
