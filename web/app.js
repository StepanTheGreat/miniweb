
function assert(expr, message = "Assertion error") {
    if (!expr) {
        throw new Error(message)
    }
}

let instance;
let memory;

const canvas = document.createElement("canvas")
document.body.appendChild(canvas);

const ctx = canvas.getContext("webgl2");
if (ctx === null) {
    alert("This page needs webgl2 support and thus can't be run");
    throw new Error("Failed to obtain a webgl2 context");
} 

const env = {
    js_println: function(start, length) {
        const view = new Uint8Array(memory.buffer, start, length);
        var enc = new TextDecoder("utf-8");

        console.log(enc.decode(view));
    },
    js_alert: function(start, length) {
        const view = new Uint8Array(memory.buffer, start, length);
        var enc = new TextDecoder("utf-8");

        alert(enc.decode(view));
    },

    js_panic: function(errPtr, errLen, filePtr, fileLen, line) {
        const enc = new TextDecoder("utf-8");

        let message = "";
        if (errLen > 0) {
            const errView = new Uint8Array(memory.buffer, errPtr, errLen);
            message = enc.decode(errView);
        }

        const fileView = new Uint8Array(memory.buffer, filePtr, fileLen);
        const filePath = enc.decode(fileView);

        message = `Caught a panic in ${filePath} at line ${line} :\n${message}`;

        alert(message);
        throw new Error(message);
    },

    glClear: function(mask) {
        ctx.clear(mask);
    },

    glClearColor: function(red, green, blue, alpha) {
        ctx.clearColor(red, green, blue, alpha)
    },
};

WebAssembly.instantiateStreaming(fetch("./web/app.wasm"), { env }).then(
    (results) => {
        instance = results.instance;
        memory = instance.exports.memory;

        // Start the main program
        instance.exports.__main();

        function draw() {
            instance.exports.__draw();
            requestAnimationFrame(draw);
        }
        requestAnimationFrame(draw);
    },
);