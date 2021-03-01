import * as wasm from "wasm-julia";

function run() {
    var data = wasm.gen(
        document.getElementById("palette-length").value,
        document.getElementById("palette-hue").value,
        document.getElementById("cx").value,
        document.getElementById("cy").value
    );
    var c = document.getElementById("myCanvas");
    var ctx = c.getContext("2d");
    var iData = new ImageData(new Uint8ClampedArray(data.buffer), 1000, 1000);
    ctx.putImageData(iData, 0, 0);
}

document.getElementById("generate").onclick = run
document.getElementById("palette-hue").oninput = function() {
    document.getElementById("palette-hue-info").textContent = document.getElementById("palette-hue").value;
    run();
}

document.getElementById("cx").oninput = function() {
    document.getElementById("cx-out").textContent = document.getElementById("cx").value;
    run();
}

document.getElementById("cy").oninput = function() {
    document.getElementById("cy-out").textContent = document.getElementById("cy").value;
    run();
}

run();