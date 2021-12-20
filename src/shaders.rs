pub const VS_GLSL: &'static str = r#"
    attribute vec4 vCoord;
    attribute vec4 vColor;

    varying lowp vec4 fragColor;

    uniform mat4 m;
    uniform mat4 v;
    uniform mat4 p;

    void main() {
        gl_Position = p * v * m * vCoord;
        fragColor = vColor;
    }
"#;

pub const FS_GLSL: &'static str = r#"
    varying lowp vec4 fragColor;

    void main() {
        gl_FragColor = fragColor;
    }
"#;

