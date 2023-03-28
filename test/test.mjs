import test from 'node:test';
import assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import swc from '@swc/core';

const fixturesPath = new URL(
  path.join(path.dirname(import.meta.url), 'fixtures')
).pathname;

const getConfig = (extname, swcrc = {}, config = {}) => ({
  ...swcrc,
  jsc: {
    ...swcrc?.jsc,
    parser: {
      syntax:
        extname === '.ts' || extname === '.tsx' ? 'typescript' : 'ecmascript',
      tsx: extname === '.tsx',
      jsx: extname === '.jsx' || extname === '.js',
      ...swcrc?.jsc?.parser,
    },
    transform: {
      ...swcrc?.jsc?.transform,
      react: {
        runtime: 'classic',
        pragma: 'createElement',
        pragmaFrag: 'Fragment',
        ...swcrc?.jsc?.transform?.react,
      },
    },
    experimental: {
      plugins: [
        [path.join(process.cwd(), 'swc_plugin_import_jsx_pragma.wasm'), config],
      ],
    },
  },
});

const updateSnapshot = process.env.UPDATE === 'true';

fs.readdirSync(fixturesPath).forEach((fixture) => {
  test(fixture, () => {
    const fixturePath = path.join(fixturesPath, fixture);
    const files = fs.readdirSync(fixturePath);
    const inputPath = path.join(
      fixturePath,
      files.find((file) => path.basename(file, path.extname(file)) === 'input')
    );
    const extname = path.extname(inputPath);
    const hasConfig = files.includes('config.json');
    const config = hasConfig
      ? JSON.parse(
          fs.readFileSync(path.join(fixturePath, 'config.json'), 'utf-8')
        )
      : {};
    const hasSWCRC = files.includes('.swcrc');
    const swcrc = hasSWCRC
      ? JSON.parse(fs.readFileSync(path.join(fixturePath, '.swcrc'), 'utf-8'))
      : {};

    const { code } = swc.transformFileSync(
      inputPath,
      getConfig(extname, swcrc, config)
    );

    const hasOutput = files.includes('output.js');
    if (!hasOutput && !updateSnapshot) {
      throw new Error('No output file found');
    }

    const outputPath = path.join(fixturePath, 'output.js');

    if (updateSnapshot) {
      fs.writeFileSync(outputPath, code, 'utf-8');
    } else {
      const output = fs.readFileSync(outputPath, 'utf-8');
      assert.strictEqual(code, output);
    }
  });
});
