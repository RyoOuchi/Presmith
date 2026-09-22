#!/usr/bin/env node
// Real setup/browser integration; uses existing downloaded browser/npm cache offline.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {cp, mkdtemp, mkdir, readFile, writeFile, realpath, lstat, readdir, rename, rm, chmod} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const binary = path.resolve(process.env.PRESMITH_BIN || path.join(repo, 'target/debug/presmith'));
const runtime = path.resolve(process.argv[2] || path.join(repo, 'examples/product/tooling/renderer'));
const temp = await mkdtemp(path.join(tmpdir(), 'presmith cache test '));
const cache = path.join(temp, 'shared cache');
const first = path.join(temp, 'first deck'), second = path.join(temp, 'second deck');
const env = {...process.env, PRESMITH_CACHE_DIR: cache, npm_config_offline: 'true'};
const results = [];
async function run(args, expected = 0, overrides = {}) {
  const child = spawn(binary, args, {env: {...env, ...overrides}, cwd: repo, stdio: ['ignore', 'pipe', 'pipe']});
  let stdout = '', stderr = '';
  child.stdout.on('data', x => stdout += x); child.stderr.on('data', x => stderr += x);
  const timer = setTimeout(() => child.kill('SIGTERM'), 90000);
  const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
  clearTimeout(timer);
  assert.equal(code, expected, `${args.join(' ')}\n${stdout}\n${stderr}`);
  return {stdout, stderr};
}
async function test(name, action) {
  await action(); results.push(name); console.log(`PASS ${name}`);
}
async function sources(project) {
  const files = ['deck.json', 'tooling/renderer/package.json', 'tooling/renderer/package-lock.json', 'tooling/renderer/render.mjs', 'tooling/renderer/pptx.mjs'];
  for (const dir of ['slides', 'styles', 'scripts', 'lib']) {
    for (const name of await readdir(path.join(project, dir))) files.push(`${dir}/${name}`);
  }
  return Object.fromEntries(await Promise.all(files.map(async f => [f, (await readFile(path.join(project, f))).toString('base64')])));
}
try {
  let before, shared;
  await test('legacy project-local install migrates to a shared cache without browser downloads', async () => {
    await run(['init', first]);
    before = await sources(first);
    await mkdir(cache);
    await cp(path.join(runtime, '.npm-cache'), path.join(cache, 'npm'), {recursive: true});
    for (const name of ['node_modules', '.browsers']) {
      await cp(path.join(runtime, name), path.join(first, 'tooling/renderer', name), {recursive: true, verbatimSymlinks: true});
    }
    const r = await run(['setup', first]);
    assert.match(r.stderr, /Installed shared renderer/);
    assert.doesNotMatch(r.stderr, /Downloading/);
    const modules = path.join(first, 'tooling/renderer/node_modules');
    assert.ok((await lstat(modules)).isSymbolicLink());
    shared = path.dirname(await realpath(modules));
    assert.equal(path.dirname(await realpath(path.join(first, 'tooling/renderer/.browsers'))), shared);
    assert.deepEqual(await sources(first), before);
  });
  const guard = path.join(temp, 'no npm');
  await mkdir(guard); await writeFile(path.join(guard, 'npm'), '#!/bin/sh\necho "ERROR: npm must not run for a warm cache" >&2\nexit 97\n');
  await chmod(path.join(guard, 'npm'), 0o755);
  const noNpm = {PATH: `${guard}${path.delimiter}${process.env.PATH}`};
  await test('new deck and repeated setup reuse the cache without invoking npm', async () => {
    await run(['init', second]);
    // The existing browser suite exercises the product deck's interactive lab slide.
    for (const name of ['deck.json', 'slides', 'styles', 'scripts', 'assets', 'lib']) {
      await cp(path.join(repo, 'examples/product', name), path.join(second, name), {recursive: true});
    }
    for (const deck of [second, first, second]) {
      const r = await run(['setup', deck], 0, noNpm);
      assert.match(r.stderr, /Reusing shared renderer.*no downloads/);
      assert.doesNotMatch(r.stderr, /Installing pinned|Downloading/);
      assert.equal(path.dirname(await realpath(path.join(deck, 'tooling/renderer/node_modules'))), shared);
    }
    assert.equal((await readdir(path.join(cache, 'renderer-v1'))).length, 1);
  });
  await test('both decks launch shared Chromium and retain PPTX capability', async () => {
    for (const deck of [first, second]) {
      const result = JSON.parse((await run(['doctor', deck, '--json'])).stdout);
      assert.equal(result.success, true);
      assert.equal(result.capabilities.pptx_export, true);
    }
  });
  await test('broken cache fails actionably without replacing project links or source', async () => {
    const pkg = path.join(shared, 'node_modules/playwright');
    await rename(pkg, `${pkg}.test-backup`);
    try {
      const r = await run(['setup', first], 2, noNpm);
      assert.match(r.stderr, /Renderer cache is incomplete/);
      assert.equal(path.dirname(await realpath(path.join(first, 'tooling/renderer/node_modules'))), shared);
      assert.deepEqual(await sources(first), before);
    } finally { await rename(`${pkg}.test-backup`, pkg); }
  });
  await test('local opt-out detaches one deck without changing the shared installation', async () => {
    await cp(path.join(cache, 'npm'), path.join(first, 'tooling/renderer/.npm-cache'), {recursive: true});
    await run(['setup', first, '--local']);
    for (const name of ['node_modules', '.browsers']) {
      assert.equal((await lstat(path.join(first, 'tooling/renderer', name))).isSymbolicLink(), false);
      assert.equal(path.dirname(await realpath(path.join(second, 'tooling/renderer', name))), shared);
    }
    assert.equal(JSON.parse((await run(['doctor', second, '--json'])).stdout).success, true);
    assert.deepEqual(await sources(first), before);
  });
  await test('existing browser and export regression suite passes using the shared renderer', async () => {
    const child = spawn(process.execPath, [path.join(repo, 'tests/browser.mjs'), second], {cwd: repo, env: {...env, PRESMITH_BIN: binary}, stdio: 'inherit'});
    const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
    assert.equal(code, 0);
  });
  await mkdir(path.join(repo, 'verification'), {recursive: true});
  await writeFile(path.join(repo, 'verification/cache-results.json'), JSON.stringify({success: true, platform: process.platform, tests: results, network: 'npm offline; installed browser files reused'}, null, 2));
  console.log(`PASS ${results.length} shared-cache integration checks`);
} finally { await rm(temp, {recursive: true, force: true}); }
