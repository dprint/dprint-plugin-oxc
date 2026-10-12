# dprint-plugin-oxc

[![npm version](https://img.shields.io/npm/v/@dprint/oxc.svg)](https://www.npmjs.com/package/@dprint/oxc) [![CI](https://github.com/dprint/dprint-plugin-oxc/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-oxc/actions?query=workflow%3ACI)

Adapter for [Oxc](https://github.com/oxc-project/oxc) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Supported languages

The plugin formats the languages that Oxc's formatter (oxfmt) formats natively in Rust:

- JavaScript and TypeScript (including JSX)
- JSON, JSONC, and JSON5
- CSS, SCSS, and Less
- GraphQL
- YAML
- TOML

Code embedded in these is formatted too (ex. CSS or GraphQL in a JS template literal, or YAML front matter in CSS).

Markdown is also supported, though oxfmt does not use Oxc's Markdown formatter yet. Code blocks and front matter in Markdown files are not formatted.

If you only want to use this plugin for some of these languages, [disable the other languages](#disabling-languages).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint add oxc
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

Like in oxfmt, the configuration is shared between the languages. For example, `quoteStyle` applies to JavaScript, CSS, YAML, and so on.

### Disabling languages

Every language is formatted by default. To have the plugin leave the files of a language alone, set its `<language>.enabled` option to `false`:

```jsonc
{
  "oxc": {
    "markdown.enabled": false,
    "yaml.enabled": false,
  },
}
```

The languages are `javascript`, `typescript`, `json`, `jsonc`, `json5`, `graphql`, `css`, `scss`, `less`, `yaml`, `markdown`, and `toml`. JSX is part of `javascript` and TSX is part of `typescript`.

A disabled language's files are no longer matched by this plugin, so dprint formats them with the next plugin that matches them, if any. This only decides which files are formatted: code embedded in a file of an enabled language is still formatted (ex. CSS in a JS template literal when `css.enabled` is `false`).

Note: The plugin does not understand Oxc's configuration file because it runs sandboxed in a Wasm runtime—it has no access to the file system in order to read Oxc's config.

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/oxc)

## Versioning

This repo automatically upgrades to the latest version of Oxc once a day. You can check which version of Oxc is being used by looking at the `tag` property in the `oxc_formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-oxc/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Oxc.
