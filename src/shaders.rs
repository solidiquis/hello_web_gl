pub const VS_GLSL: &'static str = r#"
    attribute vec4 vCoord;

    uniform mat4 m;
    uniform mat4 v;
    uniform mat4 p;

    void main() {
        gl_Position = p * v * m * vCoord;
    }
"#;

pub const FS_GLSL: &'static str = r#"
    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

