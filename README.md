# dprint-plugin-oxc

[![CI](https://github.com/dprint/dprint-plugin-oxc/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-oxc/actions?query=workflow%3ACI)

Adapter for [Oxc](https://github.com/oxc-project/oxc) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint config add oxc
```

Note: You do not need Oxc installed globally as dprint will run the Oxc formatter from the .wasm file in a sandboxed environment.

## Configuration

To add configuration, specify an `"oxc"` key in your dprint.json:

```jsonc
{
  "oxc": {
    "indentStyle": "space",
    "lineWidth": 100,
    "indentWidth": 2,
  },
  "plugins": [
    // ...etc...
  ],
}
```

For an overview of the config, see https://dprint.dev/plugins/oxc/config/

Note: The plugin does not understand Oxc's configuration file because it runs sandboxed in a Wasm runtime—it has no access to the file system in order to read Oxc's config.

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/oxc)

## Versioning

This repo automatically upgrades to the latest version of Oxc once a day. You can check which version of Oxc is being used by looking at the `tag` property in the `oxc_formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-oxc/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Oxc.
