# `swc-plugin-import-jsx-pragma`

[SWC](https://swc.rs/) plugin for automatically injecting an import statement for JSX pragma in classic runtime.

## Installation

```sh
npm i -D swc-plugin-import-jsx-pragma
```

## Usage

See [`jsc.experimental.plugins`](https://swc.rs/docs/configuration/compilation#jscexperimentalplugins):

```json5
// .swcrc
{
  "jsc": {
    "transform": {
      "react": {
        // Currently, these are the required configs.
        "runtime": "classic",
        "pragma": "createElement",
        "pragmaFrag": "Fragment",
      },
    },
    "experimental": {
      "plugins": [
        ["swc-plugin-import-jsx-pragma", {}]
      ]
    }
  }
}
```

It will take this input:
```js
export default function App() {
  return <h1>Hello World</h1>
}
```

And generate this output:
```js
import { createElement } from "react";
export default function App() {
    return /*#__PURE__*/ createElement("h1", null, "Hello World");
}

```

## Options

- `importSource`: `string`, defaults to `react`.
