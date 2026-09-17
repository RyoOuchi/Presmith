import {createInterface} from 'node:readline';
import {mkdir, writeFile} from 'node:fs/promises';
import path from 'node:path';
import {inspectSlide} from './diagnostics.mjs';

let browser, closing = false;
async function close() { if (closing) return; closing = true; try { await browser?.close(); } catch {} }
for (const signal of ['SIGINT','SIGTERM']) process.on(signal, async () => { await close(); process.exit(130); });
const lines = createInterface({input:process.stdin});
const input = await new Promise(resolve => lines.once('line', line => resolve(JSON.parse(line))));
lines.on('line', async () => { await close(); process.exit(130); });
// Closing the parent pipe (including a killed Rust CLI) tears down the browser.
lines.on('close', async () => { if (!closing) { await close(); process.exit(130); } });
let result = {schema_version:1, command:input.action, success:false, findings:[], artifacts:[], error:null};
function escape(s) { return String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('"','&quot;'); }
try {
  const {chromium} = await import('playwright');
  try { browser = await chromium.launch({headless:true, timeout:20000}); }
  catch (error) { throw new Error(`Chromium cannot launch. Run decksmith setup in this project. On Linux, install the Playwright system libraries. ${error.message}`); }
  if (input.action === 'probe') {
    result.success = true; result.runtime = {node:process.version, chromium:browser.version()};
  } else {
    const m = input.manifest;
    const context = await browser.newContext({viewport:{width:m.width,height:m.height},deviceScaleFactor:1,reducedMotion:'reduce',serviceWorkers:'block'});
    context.setDefaultTimeout(15000);
    const origin = new URL(input.url).origin;
    const page = await context.newPage();
    let active = null;
    const requests = [], errors = [];
    await context.route('**/*', async route => {
      const url = new URL(route.request().url());
      if (url.origin === origin || ['data:','blob:'].includes(url.protocol)) await route.continue();
      else { requests.push({url:url.href, reason:'External network request blocked', slide:active}); await route.abort('blockedbyclient'); }
    });
    page.on('pageerror', error => errors.push({message:error.message,slide:active}));
    page.on('requestfailed', request => requests.push({url:request.url(),reason:request.failure()?.errorText || 'Request failed',slide:active}));
    page.on('response', response => { if (response.status() >= 400) requests.push({url:response.url(),reason:`HTTP ${response.status()}`,slide:active}); });
    await page.goto(input.url, {waitUntil:'domcontentloaded',timeout:20000});
    await page.waitForFunction(() => window.Decksmith?.version === 1);
    await page.evaluate(() => Decksmith.setExportMode(true));
    const slides = await page.evaluate(() => Decksmith.listSlides());
    if (slides.map(s => s.id).join('\0') !== m.slides.map(s => s.id).join('\0')) throw new Error('Runtime slide order differs from manifest');
    const selected = input.slide ? slides.filter(s => s.id === input.slide) : slides;
    if (!selected.length) throw new Error(`Unknown slide: ${input.slide}`);
    const captures = [];
    for (const slide of selected) {
      active = slide.id;
      await page.evaluate(id => Decksmith.select(id), slide.id);
      try { await page.evaluate(id => Decksmith.ready(id), slide.id); }
      catch (error) { result.findings.push({severity:'error',rule_id:'runtime.not_ready',slide_id:slide.id,element_id:null,source:slide.source,message:error.message,measurements:{timeout_ms:10000}}); }
      if (input.action === 'check') result.findings.push(...await page.evaluate(inspectSlide, {id:slide.id,source:slide.source}));
      // Exports/rendering also reject broken images, required assets, and JS failures.
      else result.findings.push(...(await page.evaluate(inspectSlide, {id:slide.id,source:slide.source})).filter(f => f.rule_id === 'asset.broken_image'));
      if (input.action === 'render') {
        await mkdir(path.join(input.out,'slides'), {recursive:true});
        const name = `slides/${slide.id}.png`;
        const png = await page.locator(`[data-slide-id="${slide.id}"]`).screenshot({path:path.join(input.out,name),animations:'disabled',scale:'css',timeout:15000});
        captures.push({slide,png}); result.artifacts.push({kind:'slide_png',slide_id:slide.id,path:name,width:m.width,height:m.height});
      }
    }
    // Exercise actual browser navigation in the portable HTML contract.
    if (input.action === 'html') {
      await page.evaluate(() => Decksmith.setExportMode(false));
      await page.evaluate(id => Decksmith.select(id), slides[0].id);
      for (let i = 1; i < slides.length; i++) {
        await page.keyboard.press('ArrowRight');
        const id = await page.evaluate(() => Decksmith.current());
        if (id !== slides[i].id || new URL(page.url()).hash !== `#${id}`) throw new Error('HTML keyboard/hash navigation failed');
      }
    }
    for (const request of requests) {
      // Find a slide-local source when the initial page load requested an image on a hidden slide.
      const element = await page.evaluate(url => {
        const el = [...document.querySelectorAll('[src],[href]')].find(e => (e.src || e.href) === url);
        const slide = el?.closest('[data-slide-id]');
        return {id:el?.dataset.elementId || el?.id || null, slide:slide?.dataset.slideId || null, source:slide?.dataset.source || null};
      }, request.url);
      if (input.slide && element.slide && element.slide !== input.slide) continue;
      result.findings.push({severity:'error',rule_id:'asset.request_failed',slide_id:element.slide || request.slide,element_id:element.id,source:element.source || (() => {try{return decodeURIComponent(new URL(request.url).pathname.slice(1));}catch{return null;}})(),message:`${request.reason}: ${request.url}`,measurements:{url:request.url}});
    }
    for (const error of errors) {
      const source = m.slides.find(s => s.id === error.slide)?.source || 'scripts/';
      result.findings.push({severity:'error',rule_id:'runtime.javascript',slide_id:error.slide,element_id:null,source,message:error.message,measurements:{}});
    }
    result.findings = result.findings.filter((f,i,all) => all.findIndex(x => x.rule_id === f.rule_id && x.slide_id === f.slide_id && x.element_id === f.element_id && x.message === f.message) === i);
    const hasErrors = result.findings.some(f => f.severity === 'error');
    if (!hasErrors) {
      if (input.action === 'pdf') {
        const {PDFDocument} = await import('pdf-lib');
        const pdf = await page.pdf({width:`${m.width}px`,height:`${m.height}px`,printBackground:true,displayHeaderFooter:false,preferCSSPageSize:true,margin:{top:0,bottom:0,left:0,right:0},tagged:true});
        const parsed = await PDFDocument.load(pdf);
        if (parsed.getPageCount() !== m.slides.length) throw new Error(`PDF page count ${parsed.getPageCount()} != ${m.slides.length}; check print CSS`);
        await writeFile(path.join(input.out,'deck.pdf'),pdf);
        result.artifacts.push({kind:'pdf',path:'deck.pdf',pages:parsed.getPageCount()});
      }
      if (input.action === 'html') result.artifacts.push({kind:'html',path:'',slides:slides.length,external_network:'blocked',navigation_verified:true});
      if (input.action === 'render') {
        const sheet = await context.newPage();
        const cellWidth = 480, imageHeight = cellWidth * m.height / m.width;
        await sheet.setViewportSize({width:1024,height:Math.ceil(selected.length/2)*(imageHeight+64)+72});
        await sheet.setContent(`<html><head><style>*{box-sizing:border-box}body{margin:0;padding:24px;background:#e6e9e5;color:#203039;font:16px -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}h1{font-size:20px;margin:0 0 20px;font-weight:600}.grid{display:grid;grid-template-columns:repeat(2,480px);gap:24px 16px}figure{margin:0}img{display:block;width:480px;height:${imageHeight}px}figcaption{font-size:14px;padding-top:8px}</style></head><body><h1>${escape(m.title)} · ${selected.length} slides</h1><div class="grid">${captures.map(({slide,png})=>`<figure><img src="data:image/png;base64,${png.toString('base64')}"><figcaption>${String(slide.index+1).padStart(2,'0')} / ${escape(slide.id)} — ${escape(slide.title || '')}</figcaption></figure>`).join('')}</div></body></html>`);
        await sheet.evaluate(async () => { await document.fonts.ready; await Promise.all([...document.images].map(i => i.decode())); });
        await sheet.screenshot({path:path.join(input.out,'contact-sheet.png'),fullPage:true});
        await sheet.close();
        result.artifacts.push({kind:'contact_sheet',path:'contact-sheet.png'});
      }
      result.success = true;
    } else {
      result.artifacts = [];
      result.error = {kind:'deck',message:'Deck problems found; inspect findings and rerun.'};
    }
    await context.close();
  }
} catch (error) {
  result.artifacts = [];
  result.error = {kind:'operational',message:error.message};
} finally {
  await close();
  process.stdout.write(`${JSON.stringify(result)}\n`);
  lines.close(); process.stdin.destroy();
}
