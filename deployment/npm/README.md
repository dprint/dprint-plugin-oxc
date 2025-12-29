# @dprint/oxc

npm distribution of [dprint-plugin-oxc](https://github.com/dprint/dprint-plugin-oxc) which is an adapter plugin for [Oxc](https://github.com/oxc-project/oxc).

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or just use @dprint/formatter and download the [dprint-plugin-oxc Wasm file](https://github.com/dprint/dprint-plugin-oxc/releases).

## Example

```ts
import { createFromBuffer } from "@dprint/formatter";
import { getBuffer } from "@dprint/oxc";

const formatter = createFromBuffer(getBuffer());

console.log(
  formatter.formatText("test.js", "console.log(  5  )"),
);
```
