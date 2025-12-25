const PAGE_SIZE = Math.pow(2, 16);

function assert(expr, message = "Assertion error") {
    if (!expr) {
        throw new Error(message)
    }
}

let instance;
let memory;

const encoder = new TextDecoder("utf-8");
const canvas = document.createElement("canvas")
document.body.appendChild(canvas);

const ctx = canvas.getContext("webgl2");
if (ctx === null) {
    alert("This page needs webgl2 support and thus can't be run");
    throw new Error("Failed to obtain a webgl2 context");
} 

const env = {
    js_request_pages(pages) {
        memory.grow(pages);
    },

    js_allocated_pages() {
        return memory.buffer.byteLength / PAGE_SIZE
    },

    js_println(start, length) {
        const view = new Uint8Array(memory.buffer, start, length);

        console.log(encoder.decode(view));
    },

    js_println_number(num) {
        console.log(num);
    },

    js_alert(start, length) {
        const view = new Uint8Array(memory.buffer, start, length);
        alert(encoder.decode(view));
    },

    js_panic(errPtr, errLen, filePtr, fileLen, line) {
        let message = "";
        if (errLen > 0) {
            const errView = new Uint8Array(memory.buffer, errPtr, errLen);
            message = encoder.decode(errView);
        }

        const fileView = new Uint8Array(memory.buffer, filePtr, fileLen);
        const filePath = encoder.decode(fileView);

        message = `Caught a panic in ${filePath} at line ${line} :\n${message}`;

        throw new Error(message);
    },

    glClear(mask) {
        ctx.clear(mask);
    },

    glClearColor(red, green, blue, alpha) {
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