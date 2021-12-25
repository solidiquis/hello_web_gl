const rust = import("./pkg/index");
const canvas = document.getElementById("hello-webgl");
const gl = canvas.getContext('webgl', { antialias: true });

window.onload = rust.then(wasm => {
  if (!gl) {
    alert("Failed to initialize WebGL");
    return
  }

  try {
    let helloWebGL = new wasm.HelloWebGL("hello-webgl");
    helloWebGL.render(canvas.width, canvas.height);
  } catch(e) {
    console.error(e);
  }
});
