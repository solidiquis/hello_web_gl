pub const VS_GLSL: &'static str = r#"
    attribute vec3 position;
    attribute vec3 color;

    varying lowp vec4 fragColor;

    uniform mat4 m;
    uniform mat4 v;
    uniform mat4 p;

    void main() {
        gl_Position = p * v * m * vec4(position, 1.0);
        fragColor = vec4(color, 1.0);
    }
"#;

pub const FS_GLSL: &'static str = r#"
    varying lowp vec4 fragColor;

    void main() {
        gl_FragColor = fragColor;
    }
"#;

