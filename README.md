<div align="center">
  <h3 align="center">WebPL</h3>

  <p align="center">
    A Prolog interpreter for the browser<br>
    <a href="https://webpl.whenderson.dev"><strong>Try Now »</strong></a>
  </p>
</div>

<hr><br>

WebPL is a web-native Prolog implementation, written from scratch in Rust and compiled to WebAssembly. Alongside the core interpreter, it also includes a simple [web-based IDE](https://webpl.whenderson.dev) for writing and running Prolog code in the browser, a precise garbage collector, and a JavaScript FFI for calling JavaScript functions from Prolog.

This project is the basis of my Cambridge Computer Science Tripos dissertation, written in the 2024-2025 academic year, and can be found [here](https://github.com/w-henderson/WebPL/blob/main/dissertation.pdf).

## Installation

WebPL is available on NPM and can be installed with the following command:

```bash
npm install webpl
```

## Usage

A basic usage example is shown below.

```js
import init, { Solver } from "webpl";

async function main() {
  await init();
  const solver = await Solver.solve("is_even(X) :- 0 is X mod 2.", "is_even(4).");
  console.log(await solver.next());
}
```