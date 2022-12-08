
const node_process = require("./");
console.log("-");
console.log("[WASM]: Running init test")
let v = node_process.add(1,2);
console.log("v:",v);
node_process.test();

