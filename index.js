const rust = import("./pkg/index");
const canvas = document.getElementById("hello-webgl");
const gl = canvas.getContext('webgl', { antialias: true });

window.onload = rust.then(wasm => {
  if (!gl) {
    alert("Failed to initialize WebGL");
    return
  }

  wasm.hello_webgl()
});
