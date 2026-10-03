// @ts-check
const assert = require("assert");
const createFromBuffer = require("@dprint/formatter").createFromBuffer;
const getBuffer = require("./index").getBuffer;

const formatter = createFromBuffer(getBuffer());
const result = formatter.formatText({
  filePath: "file.js",
  fileText: "console.log (   5 )",
});

assert.strictEqual(result, "console.log(5);\n");

// ensure the other formatters work in the Wasm build
for (
  const [filePath, fileText, expected] of [
    ["file.json", "{\"a\":1}", "{ \"a\": 1 }\n"],
    ["file.css", "a{b:c}", "a {\n  b: c;\n}\n"],
    ["file.graphql", "{a}", "{\n  a\n}\n"],
    ["file.yaml", "a:   1", "a: 1\n"],
    ["file.toml", "a=1", "a = 1\n"],
    ["file.ts", "const a = css`a{b:c}`;", "const a = css`\n  a {\n    b: c;\n  }\n`;\n"],
  ]
) {
  assert.strictEqual(formatter.formatText({ filePath, fileText }), expected, filePath);
}
