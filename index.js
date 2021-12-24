const rust = import("./pkg/index");
const canvas = document.getElementById("hello-webgl");
const gl = canvas.getContext('webgl', { antialias: true });

window.onload = rust.then(wasm => {
  if (!gl) {
    alert("Failed to initialize WebGL");
    return
  }

  try {
    wasm.hello_webgl();
  } catch(e) {
    console.error(e);
  }
});
