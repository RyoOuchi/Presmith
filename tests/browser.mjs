#!/usr/bin/env node
// End-to-end verification against the compiled CLI. No test-only CLI features.
import assert from 'node:assert/strict';
import {spawn, execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {once} from 'node:events';
import {createServer} from 'node:net';
import {mkdtemp, mkdir, readFile, writeFile, readdir, symlink, rm, cp, realpath} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const binary = path.resolve(process.env.PRESMITH_BIN || path.join(repo,'target/debug/presmith'));
const runtimeProject = path.resolve(process.argv[2] || path.join(repo,'examples/product'));
const runtime = path.join(runtimeProject,'tooling/renderer');
process.env.PLAYWRIGHT_BROWSERS_PATH = path.join(runtime,'.browsers');
const require = createRequire(path.join(runtime,'package.json'));
const {chromium} = require('playwright');
const {PDFDocument} = require('pdf-lib');
const JSZip = require('jszip');
const temp = await mkdtemp(path.join(tmpdir(),'presmith integration '));
const project = path.join(temp,'a talk with spaces');
const results = [], children = new Set(), observedPids = new Set();
let browser;
const pause = ms => new Promise(r=>setTimeout(r,ms));
function descendants(pid) {
  if (process.platform === 'win32') return [];
  const rows = execFileSync('ps',['-axo','pid=,ppid='],{encoding:'utf8'}).trim().split('\n').map(l=>l.trim().split(/\s+/).map(Number));
  const pids = new Set([pid]); let count;
  do { count=pids.size; for (const [p,pp] of rows) if (pids.has(pp)) pids.add(p); } while (count !== pids.size);
  return [...pids];
}
function start(args,cwd=temp,command=binary) {
  const child=spawn(command,args,{cwd,stdio:['ignore','pipe','pipe']}); children.add(child);
  let stdout='',stderr=''; child.stdout.on('data',d=>stdout+=d); child.stderr.on('data',d=>stderr+=d);
  const done=once(child,'exit').then(([code,signal])=>{children.delete(child);return {code,signal,stdout,stderr};});
  return {child,done,get stdout(){return stdout;},get stderr(){return stderr;}};
}
async function run(args,code=0) {
  const command=start(args);
  const monitor=setInterval(()=>{try{descendants(command.child.pid).forEach(p=>observedPids.add(p));}catch{}},80);
  const timer=setTimeout(()=>command.child.kill('SIGTERM'),90000);
  const result=await command.done; clearInterval(monitor); clearTimeout(timer);
  assert.equal(result.code,code,`${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  return args.includes('--json') ? JSON.parse(result.stdout) : result;
}
async function until(f, label, ms=10000) { const start=Date.now(); while(Date.now()-start<ms) {if(await f())return;await pause(80);}throw new Error(`Timed out: ${label}`); }
async function test(name,fn) { const start=Date.now(); await fn(); results.push({name,success:true,ms:Date.now()-start}); console.log(`PASS ${name}`); }
async function json() { return JSON.parse(await readFile(path.join(project,'deck.json'),'utf8')); }
async function manifest(m) { await writeFile(path.join(project,'deck.json'),JSON.stringify(m,null,2)); }
async function hashes() { const map={}; for(const file of await readdir(path.join(project,'slides'))) map[file]=createHash('sha256').update(await readFile(path.join(project,'slides',file))).digest('hex');return map; }
try {
  await test('compiled binary initializes outside repository, including spaces',async()=>{
    await run(['init',project]);
    await run(['init',project],2);
    for(const name of ['node_modules','.browsers']) await symlink(path.join(runtime,name),path.join(project,'tooling/renderer',name),'dir');
  });
  await test('doctor launches Chromium and returns JSON capabilities',async()=>{
    const r=await run(['doctor',project,'--json']);assert.equal(r.success,true);assert.equal(r.capabilities.pdf_export,true);assert.equal(r.capabilities.pptx_export,true);
  });
  browser=await chromium.launch({headless:true});
  await test('preview: navigation, scaling, isolation, reload, error recovery and shutdown',async()=>{
    const preview=start(['dev',project,'--port','0']);
    await until(()=>preview.stderr.includes('Preview:'),'preview URL');
    const url=preview.stderr.match(/http:\/\/127\.0\.0\.1:\d+\//)[0];
    const context=await browser.newContext({viewport:{width:900,height:700}}), page=await context.newPage();
    await page.goto(url);await page.waitForFunction(()=>window.Decksmith?.current()==='intro');
    assert.equal(await page.locator('[data-slide-id="workflow"]').evaluate(e=>e.hidden&&e.inert),true);
    await page.keyboard.press('ArrowRight'); assert.equal(await page.evaluate(()=>Decksmith.current()),'workflow');
    assert.equal(new URL(page.url()).hash,'#workflow');
    assert.ok(await page.locator('#deck-stage').evaluate(e=>e.getBoundingClientRect().width<=900));
    for(const forbidden of ['deck.json','AGENTS.md','tooling/renderer/package.json','.git/config','slides/intro.html','../Cargo.toml','%2e%2e/Cargo.toml']) assert.equal((await fetch(new URL(forbidden,url))).status,404,forbidden);
    await Promise.all(Array.from({length:8},async()=>{const p=await context.newPage();await p.goto(url,{waitUntil:'load',timeout:10000});await p.close();}));
    const source=path.join(project,'slides/workflow.html');const before=await readFile(source,'utf8');
    await writeFile(source,before.replace('Three moves. One clear message.','A focused revision.'));
    await page.waitForFunction(()=>document.querySelector('[data-slide-id="workflow"] h2')?.textContent==='A focused revision.');assert.equal(new URL(page.url()).hash,'#workflow');
    const deck=await readFile(path.join(project,'deck.json'));await writeFile(path.join(project,'deck.json'),'{');
    await page.waitForFunction(()=>document.getElementById('deck-build-error')?.hidden === false);
    await writeFile(path.join(project,'deck.json'),deck);await page.waitForFunction(()=>document.getElementById('deck-build-error')?.hidden);
    await writeFile(source,before);
    await context.close();preview.child.kill('SIGTERM');await preview.done;
    await assert.rejects(fetch(url));
  });
  await test('starter check is clean; warning-only returns zero',async()=>{
    const r=await run(['check',project,'--json']);assert.deepEqual(r.findings,[]);
    const file=path.join(project,'slides/intro.html');const before=await readFile(file,'utf8');await writeFile(file,before+'<span data-element-id="small-only" style="position:absolute;top:10px;font-size:12px">small</span>');
    const warning=await run(['check',project,'--slide','intro','--json']);assert.ok(warning.findings.some(f=>f.rule_id==='text.too_small'));assert.equal(warning.success,true);await writeFile(file,before);
  });
  await test('screenshots: count, dimensions, manifest order, selected stable filename',async()=>{
    const r=await run(['render',project,'--json']);const slides=r.artifacts.filter(a=>a.kind==='slide_png');assert.deepEqual(slides.map(a=>a.slide_id),['intro','workflow','next']);
    for(const slide of slides) {const bytes=await readFile(slide.path);assert.equal(bytes.readUInt32BE(16),1280);assert.equal(bytes.readUInt32BE(20),720);}
    const selected=await run(['render',project,'--slide','workflow','--out',path.join(temp,'selected images'),'--json']);assert.equal(selected.artifacts.length,2);assert.equal(path.basename(selected.artifacts[0].path),'workflow.png');
    const m=await json();m.slides.reverse();await manifest(m);
    const reordered=await run(['render',project,'--json']);assert.deepEqual(reordered.artifacts.filter(a=>a.kind==='slide_png').map(a=>a.slide_id),['next','workflow','intro']);m.slides.reverse();await manifest(m);
  });
  await test('contact-sheet slide ID cannot collide with the overview filename',async()=>{
    const m=await json();m.slides[0].id='contact-sheet';await manifest(m);
    const result=await run(['render',project,'--slide','contact-sheet','--json']);
    assert.notEqual(result.artifacts[0].path,result.artifacts[1].path);
    const bytes=await readFile(result.artifacts[0].path);assert.equal(bytes.readUInt32BE(20),720);
    m.slides[0].id='intro';await manifest(m);
  });
  await test('PDF: exact page count and logical aspect ratio',async()=>{
    const result=await run(['export',project,'--format','pdf','--out',path.join(temp,'a talk.pdf'),'--json']);
    const pdf=await PDFDocument.load(await readFile(result.artifacts[0].path));assert.equal(pdf.getPageCount(),3);
    for(const page of pdf.getPages()) {const size=page.getSize();assert.ok(Math.abs(size.width-960)<1);assert.ok(Math.abs(size.height-540)<1);}
  });
  await test('PPTX: native text, shapes, images, SVG primitives, notes, order and explicit fallbacks',async()=>{
    const source=path.join(project,'slides/intro.html'), script=path.join(project,'scripts/custom.js');
    const before=await readFile(source), js=await readFile(script), original=await json();
    await cp(path.join(repo,'tests/fixtures/pptx/slide.html'),source);
    await writeFile(script,"Decksmith.register('intro',{async export(){await new Promise(r=>setTimeout(r,30)); document.getElementById('pptx-live').textContent='Ready 60%';}});");
    const m=await json();m.slides[0].notes='Speaker notes & source attribution';await manifest(m);
    const output=path.join(temp,'editable talk.pptx');
    const result=await run(['export',project,'--format','pptx','--out',output,'--json']);
    assert.equal(result.artifacts[0].path,await realpath(output));assert.equal(result.artifacts[0].slides,3);
    const stats=result.artifacts[0].editability[0];
    assert.ok(stats.text_boxes>5);assert.ok(stats.shapes>2);assert.equal(stats.images,3);assert.equal(stats.rasterized_elements,2);
    assert.ok(result.findings.some(f=>f.rule_id==='pptx.rasterized'&&f.element_id==='canvas-fallback'));
    assert.ok(result.findings.some(f=>f.rule_id==='pptx.rasterized'&&f.element_id==='explicit-fallback'));
    const zip=await JSZip.loadAsync(await readFile(output),{checkCRC32:true});
    assert.equal(Object.keys(zip.files).filter(p=>/^ppt\/slides\/slide\d+\.xml$/.test(p)).length,3);
    const xml=await zip.file('ppt/slides/slide1.xml').async('string');
    for (const text of ['Editable &amp; portable','Normal','bold','accent','Ready 60%','SVG label']) assert.ok(xml.includes(text),text);
    assert.ok(!xml.includes('HIDDEN CONTENT'));assert.ok(!xml.includes('INVISIBLE CONTENT'));assert.ok(!xml.includes('Before hook'));
    assert.ok(xml.includes('intro/native-shape/'));assert.ok(xml.includes('<p:pic>'));assert.ok(xml.includes('b="1"'));
    assert.ok(xml.includes('typeface="Arial"'));assert.ok(!xml.includes('PresmithDefinitelyMissingFont'));
    assert.ok(!xml.includes('typeface="Helvetica Neue"'));
    const notes=await zip.file('ppt/notesSlides/notesSlide1.xml').async('string');assert.ok(notes.includes('Speaker notes &amp; source attribution'));
    const presentation=await zip.file('ppt/presentation.xml').async('string');assert.match(presentation,/cx="12192000" cy="6858000"/);
    m.slides.reverse();await manifest(m);
    const reversed=await run(['export',project,'--format','pptx','--json']);assert.deepEqual(reversed.artifacts[0].editability.map(s=>s.slide_id),['next','workflow','intro']);
    // A failed export must not replace a previously successful file.
    const good=await readFile(output);
    await writeFile(source,'<img src="assets/missing.png" alt="missing">');
    const failed=await run(['export',project,'--format','pptx','--out',output,'--json'],1);assert.deepEqual(failed.artifacts,[]);assert.deepEqual(await readFile(output),good);
    await writeFile(source,before);await writeFile(script,js);await manifest(original);
  });
  await test('HTML: ordinary static server, subdirectory, navigation, no external requests/dependencies',async()=>{
    const output=path.join(temp,'static','talk');const result=await run(['export',project,'--format','html','--out',output,'--json']);assert.equal(result.artifacts[0].navigation_verified,true);
    const listing=await readdir(output,{recursive:true});assert.ok(!listing.some(p=>/node_modules|tooling|package-lock|deck.json/.test(p)));
    const socket=createServer();await new Promise(resolve=>socket.listen(0,'127.0.0.1',resolve));const port=socket.address().port;await new Promise(resolve=>socket.close(resolve));
    const server=start(['-u','-m','http.server',String(port),'--bind','127.0.0.1','--directory',path.dirname(output)],temp,'python3');
    const url=`http://127.0.0.1:${port}/talk/`;
    try {
      await until(async()=>{
        if(server.child.exitCode!==null)throw new Error(`Static server exited with ${server.child.exitCode}`);
        try{return (await fetch(url)).ok;}catch{return false;}
      },'static server');
    } catch(error) {
      throw new Error(`${error.message}\n${server.stdout}\n${server.stderr}`,{cause:error});
    }
    const context=await browser.newContext(), page=await context.newPage(), external=[];
    await context.route('**/*',route=>{if(new URL(route.request().url()).origin!==new URL(url).origin){external.push(route.request().url());return route.abort();}return route.continue();});
    await page.goto(url);await page.waitForFunction(()=>window.Decksmith?.current()==='intro');await page.keyboard.press('End');assert.equal(await page.evaluate(()=>Decksmith.current()),'next');await page.reload();assert.equal(await page.evaluate(()=>Decksmith.current()),'next');await page.keyboard.press('Home');assert.equal(await page.evaluate(()=>Decksmith.current()),'intro');assert.deepEqual(external,[]);
    await context.close();server.child.kill('SIGTERM');await server.done;
  });
  await test('broken fixtures fire the important rules; decorative overflow is exempt',async()=>{
    const source=path.join(project,'slides/workflow.html'),script=path.join(project,'scripts/custom.js');const before=await readFile(source),js=await readFile(script);
    await cp(path.join(repo,'tests/fixtures/broken/slide.html'),source);await cp(path.join(repo,'tests/fixtures/broken/script.js'),script);
    const result=await run(['check',project,'--slide','workflow','--json'],1);const rules=new Set(result.findings.map(f=>f.rule_id));
    for(const rule of ['element.duplicate_id','layout.out_of_bounds','text.overflow','text.too_small','asset.broken_image','asset.request_failed','runtime.javascript'])assert.ok(rules.has(rule),`missing ${rule}`);
    assert.ok(!result.findings.some(f=>f.element_id==='decoration'));
    for(const finding of result.findings)for(const field of ['severity','rule_id','slide_id','element_id','source','message','measurements'])assert.ok(field in finding);
    await run(['render',project,'--slide','workflow','--json'],1);
    await writeFile(source,before);await writeFile(script,js);
  });
  await test('required external requests are blocked and reported',async()=>{
    const file=path.join(project,'slides/intro.html'),before=await readFile(file);await writeFile(file,before+'<img src="https://example.invalid/missing.png" data-element-id="remote" alt="remote">');
    const result=await run(['check',project,'--slide','intro','--json'],1);assert.ok(result.findings.some(f=>f.rule_id==='asset.request_failed'&&f.message.includes('External network request blocked')));await writeFile(file,before);
  });
  await test('targeted revision preserves every unrelated slide source',async()=>{
    const before=await hashes(),file=path.join(project,'slides/workflow.html');await writeFile(file,(await readFile(file,'utf8')).replace('Three moves. One clear message.','One story. Three deliberate moves.'));
    const after=await hashes();assert.equal(after['intro.html'],before['intro.html']);assert.equal(after['next.html'],before['next.html']);assert.notEqual(after['workflow.html'],before['workflow.html']);
    await run(['check',project,'--slide','workflow','--json']);await run(['render',project,'--slide','workflow','--json']);results.push({name:'targeted-revision-evidence',before,after,success:true});
  });
  await test('Paper theme uses the same runtime and checks',async()=>{
    const m=await json();m.styles[0]='styles/paper.css';await manifest(m);await run(['check',project,'--json']);m.styles[0]='styles/theme.css';await manifest(m);
  });
  await test('registered async state is awaited and interactive state resets on export',async()=>{
    // The product deck includes the slider and deterministic 60% snapshot.
    const preview=start(['dev',runtimeProject,'--port','0']);await until(()=>preview.stderr.includes('Preview:'),'product preview');const url=preview.stderr.match(/http:\/\/127\.0\.0\.1:\d+\//)[0];
    const page=await browser.newPage();await page.goto(`${url}#lab`,{waitUntil:'domcontentloaded',timeout:15000});await page.evaluate(()=>Decksmith.ready('lab'));await page.locator('#adoption').focus();await page.keyboard.press('ArrowRight');assert.equal(await page.evaluate(()=>Decksmith.current()),'lab');
    await page.locator('#adoption').fill('80');await page.locator('#adoption').dispatchEvent('input');assert.equal(await page.locator('#adoption-label').textContent(),'80%');
    await page.evaluate(async()=>{Decksmith.setExportMode(true);await Decksmith.ready('lab');});assert.equal(await page.locator('#adoption-label').textContent(),'60%');await page.close();preview.child.kill('SIGTERM');await preview.done;
    const js=path.join(project,'scripts/custom.js');const original=await readFile(js);await writeFile(js,"Decksmith.register('intro',{async init(slide){await new Promise(r=>setTimeout(r,250)); slide.querySelector('h1').dataset.ready='yes';}, export(slide){if(slide.querySelector('h1').dataset.ready!=='yes')throw new Error('Not ready');}});");await run(['render',project,'--slide','intro','--json']);await writeFile(js,original);
  });
  await test('missing Chromium gives actionable JSON without false success',async()=>{
    const link=path.join(project,'tooling/renderer/.browsers');await rm(link);await mkdir(link);
    const r=await run(['render',project,'--json'],2);assert.equal(r.success,false);assert.match(r.error.message,/Chromium|setup/);assert.deepEqual(r.artifacts,[]);
    await rm(link,{recursive:true});await symlink(path.join(runtime,'.browsers'),link,'dir');
  });
  await test('interruption closes helper and browser descendants',async()=>{
    const js=path.join(project,'scripts/custom.js');const original=await readFile(js);await writeFile(js,"Decksmith.register('intro',{init(){return new Promise(()=>{});}});");
    const render=start(['render',project,'--json']);await pause(1200);const pids=descendants(render.child.pid);assert.ok(pids.length>=3,'expected Rust, helper and browser processes');pids.forEach(p=>observedPids.add(p));render.child.kill('SIGTERM');const result=await render.done;assert.equal(result.code,130);
    await until(()=>pids.every(pid=>{try{process.kill(pid,0);return false;}catch{return true;}}),'child cleanup');await writeFile(js,original);
  });
  await browser.close();browser=null;
  await test('successful and failed export commands leave no child processes',async()=>{
    await until(()=>[...observedPids].every(pid=>{try{process.kill(pid,0);return false;}catch{return true;}}),'all observed CLI subprocesses reaped');
  });
  await mkdir(path.join(repo,'verification'),{recursive:true});
  await writeFile(path.join(repo,'verification/browser-results.json'),JSON.stringify({success:true,platform:process.platform,tests:results},null,2));
  console.log(`${results.filter(r=>r.ms!==undefined).length} browser integration checks passed.`);
} finally {
  for(const child of children)child.kill('SIGTERM');
  await browser?.close();
  await rm(temp,{recursive:true,force:true});
}
