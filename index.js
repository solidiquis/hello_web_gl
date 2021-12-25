const rust = import("./pkg/index");
const canvas = document.getElementById("hello-webgl");
const gl = canvas.getContext('webgl', { antialias: true });

window.onload = rust.then(wasm => {
  if (!gl) {
    alert("Failed to initialize WebGL");
    return;
  }

  let helloWebGL;
  try {
    helloWebGL = new wasm.HelloWebGL("hello-webgl");
  } catch(e) {
    alert("Failed to initialize HelloWebGL.");
    console.error(e);
    return;
  }

  const FPS_THROTTLE = 1000 / 30; // 1000 ms per 30 frames
  var lastDrawTime = -1; 
  var initTime = Date.now();

  function render() {
    window.requestAnimationFrame(render);
    
    let currTime = Date.now();

    if (currTime - lastDrawTime < FPS_THROTTLE)
      return;

    lastDrawTime = currTime;

    let elapsedTime = (currTime - initTime) / 1000;

    helloWebGL.render(canvas.width, canvas.height, elapsedTime);
  }

  render();
});
