
const node_process = require("./");
console.log("-");
console.log("[WASM]: Running init test")
let v = node_process.add(1,2);
console.log("v:",v);
node_process.test();
// node_process.test_task();

/*
let {spawn} = require("node:child_process")
let cp = spawn("ls");
cp.stdout.on("data", (data)=>{
    console.log("data:", data+"")
})
cp.stderr.on("data", (data)=>{
    console.log("error:", data)
})

cp.on("disconnect", (data)=>{
    console.log("disconnect:", data)
})
*/

//setTimeout(()=>{
    //console.log("timeout");
//}, 5000)

