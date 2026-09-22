// HTML is measured in Chromium after the normal export hooks. The scene contains
// editable text/geometry plus explicit bitmap fallbacks; it is never a full-slide
// screenshot disguised as an editable presentation.
import PptxGenJS from 'pptxgenjs';
import JSZip from 'jszip';
import {writeFile} from 'node:fs/promises';

export function extractScene({id}) {
  const root = document.querySelector(`[data-slide-id="${id}"]`);
  const origin = root.getBoundingClientRect();
  const items = [], warnings = [];
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 1;
  const ctx = canvas.getContext('2d', {willReadFrequently:true});
  let serial = 0, characters = 0;
  function color(value, opacity = 1) {
    ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = value; ctx.fillRect(0, 0, 1, 1);
    const [r,g,b,a] = ctx.getImageData(0, 0, 1, 1).data;
    return {color:[r,g,b].map(n=>n.toString(16).padStart(2,'0')).join('').toUpperCase(),
      transparency:Math.round(100 * (1 - a / 255 * opacity))};
  }
  const box = r => ({x:r.x-origin.x, y:r.y-origin.y, w:r.width, h:r.height});
  const onSlide = r => r.width > 0 && r.height > 0 && r.right > origin.left && r.bottom > origin.top && r.left < origin.right && r.top < origin.bottom;
  const identifier = el => el.closest('[data-element-id]')?.dataset.elementId || el.id || el.tagName.toLowerCase();
  function warn(el, message, rule = 'pptx.rasterized') {
    warnings.push({element_id:identifier(el), rule_id:rule, message});
  }
  function bitmap(el, reason, image = false) {
    const r = el.getBoundingClientRect();
    if (!onSlide(r)) return;
    const x=Math.max(r.left,origin.left), y=Math.max(r.top,origin.top);
    const right=Math.min(r.right,origin.right), bottom=Math.min(r.bottom,origin.bottom);
    items.push({kind:'image', ...box({x,y,width:right-x,height:bottom-y}),
      name:identifier(el), fallback:!image, clip:{x,y,width:right-x,height:bottom-y}});
    if (reason) warn(el, reason);
  }
  const fontCache=new Map();
  function installedFont(family) {
    if (!fontCache.has(family)) {
      const probe='mmmmmmmmWWWWW0123456789';
      fontCache.set(family,['monospace','serif'].some(fallback=>{
        ctx.font=`72px ${fallback}`; const baseline=ctx.measureText(probe).width;
        ctx.font=`72px "${family}",${fallback}`;
        return Math.abs(ctx.measureText(probe).width-baseline)>.01;
      }));
    }
    return fontCache.get(family);
  }
  function font(s) {
    const families = s.fontFamily.split(',').map(f=>f.trim().replace(/^['"]|['"]$/g,''));
    // Portable Office fallback for CSS-only generic/system font names. Keep real
    // family names if installed in Chromium; font substitution is reported below.
    for (const family of families) {
      // These macOS faces are not reliably available to Office importers or
      // Google Slides. Preserve their sans/mono role with an Office family.
      if (['Helvetica Neue','Helvetica','SF Pro Text','SF Pro Display','.AppleSystemUIFont'].includes(family)) return 'Arial';
      if (['SFMono-Regular','SF Mono','Menlo','Monaco'].includes(family)) return 'Courier New';
      if (['monospace','ui-monospace'].includes(family)) return 'Courier New';
      if (['serif','ui-serif'].includes(family)) return 'Georgia';
      if (['sans-serif','ui-sans-serif','system-ui','-apple-system','BlinkMacSystemFont'].includes(family)) return 'Arial';
      if (installedFont(family)) return family;
    }
    return 'Arial';
  }
  function textStyle(s, opacity) {
    return {fontFace:font(s), fontSize:parseFloat(s.fontSize)*.75,
      bold:Number(s.fontWeight)>=600, italic:s.fontStyle!=='normal',
      charSpacing:(parseFloat(s.letterSpacing)||0)*.75,
      underline:s.textDecorationLine.includes('underline'),
      strike:s.textDecorationLine.includes('line-through') ? 'sngStrike' : undefined,
      ...color(s.color,opacity)};
  }
  function textNode(node, opacity, scale=1, paint=null) {
    const el=node.parentElement, s=getComputedStyle(el);
    if (s.visibility !== 'visible' || !node.textContent.trim()) return;
    const style=textStyle(s,opacity), range=document.createRange();
    style.fontSize*=scale; style.charSpacing*=scale;
    if (paint) Object.assign(style,color(paint,opacity));
    const lines=[];
    // Range geometry preserves browser wrapping, inline styling and explicit
    // line breaks. Keep complete visible line segments editable, not glyphs.
    for (let offset=0; offset<node.length;) {
      if (++characters > 200000) throw new Error('PPTX slide exceeds 200,000 text characters');
      const char=String.fromCodePoint(node.textContent.codePointAt(offset));
      range.setStart(node,offset); offset+=char.length; range.setEnd(node,offset);
      const r=range.getBoundingClientRect();
      if (!onSlide(r) || r.width<.01 || char==='\n' || char==='\r') continue;
      const last=lines.at(-1);
      if (last && Math.abs(last.y-r.y)<1 && Math.abs(last.right-r.x)<2) {
        last.text+=char; last.right=Math.max(last.right,r.right); last.height=Math.max(last.height,r.height);
      } else lines.push({text:char,x:r.x,y:r.y,right:r.right,height:r.height});
    }
    for (const line of lines) {
      let text=line.text;
      if (s.textTransform==='uppercase') text=text.toLocaleUpperCase();
      if (s.textTransform==='lowercase') text=text.toLocaleLowerCase();
      if (s.textTransform==='capitalize') text=text.replace(/\b\p{L}/gu,c=>c.toLocaleUpperCase());
      if (!text.trim()) continue;
      items.push({kind:'text',...box({x:line.x,y:line.y,width:line.right-line.x,height:line.height}),
        name:identifier(el),text,style});
    }
  }
  function shape(el, s, opacity) {
    const r=el.getBoundingClientRect();
    if (!onSlide(r)) return;
    const fill=color(s.backgroundColor,opacity);
    const radius=parseFloat(s.borderTopLeftRadius)||0;
    const ellipse=radius>=Math.min(r.width,r.height)/2 || s.borderTopLeftRadius==='50%';
    const borders=['Top','Right','Bottom','Left'].map(side=>({
      width:parseFloat(s[`border${side}Width`])||0,style:s[`border${side}Style`],
      ...color(s[`border${side}Color`],opacity)}));
    const uniform=borders.every(b=>JSON.stringify(b)===JSON.stringify(borders[0]));
    if (fill.transparency<100 || (uniform && borders[0].width>0)) {
      const border=borders[0];
      items.push({kind:'shape',...box(r),name:identifier(el),
        shape:ellipse?'ellipse':radius>0?'roundRect':'rect',radius,
        fill, line:uniform && border.width>0 ? {...border,width:border.width*.75} : {transparency:100,color:'000000',width:0}});
    }
    if (!uniform) for (let i=0;i<4;i++) {
      const b=borders[i]; if (!b.width || b.transparency===100) continue;
      const horizontal=i%2===0;
      items.push({kind:'shape',shape:'rect',name:`${identifier(el)} border`,
        x:r.x-origin.x+(i===1?r.width-b.width:0),y:r.y-origin.y+(i===2?r.height-b.width:0),
        w:horizontal?r.width:b.width,h:horizontal?b.width:r.height,
        fill:{color:b.color,transparency:b.transparency},line:{transparency:100,color:b.color,width:0}});
    }
  }
  function svg(el, opacity) {
    const supported=new Set(['svg','g','rect','circle','ellipse','line','text','tspan','title','desc']);
    const elements=[el,...el.querySelectorAll('*')];
    if (elements.some(e=>{
      const s=getComputedStyle(e), matrix=e.getScreenCTM?.();
      return !supported.has(e.localName) || (matrix && (Math.abs(matrix.b)>.001 || Math.abs(matrix.c)>.001 || matrix.a<=0 || matrix.d<=0))
        || s.filter!=='none' || s.clipPath!=='none' || /url\(/.test(s.fill+s.stroke)
        || ['markerStart','markerMid','markerEnd'].some(k=>s[k]&&s[k]!=='none');
    })) return false;
    function draw(e, alpha) {
      const s=getComputedStyle(e), matrix=e.getScreenCTM();
      alpha*=Number(s.opacity);
      if (!alpha || s.display==='none' || s.visibility!=='visible' || !matrix) return;
      const scale=matrix.a, r=e.getBoundingClientRect();
      const fill=s.fill==='none'?{color:'000000',transparency:100}:color(s.fill,alpha*Number(s.fillOpacity));
      const line={...(s.stroke==='none'?{color:'000000',transparency:100}:color(s.stroke,alpha*Number(s.strokeOpacity))),width:parseFloat(s.strokeWidth)*scale*.75};
      if (['rect','circle','ellipse'].includes(e.localName) && onSlide(r)) {
        items.push({kind:'shape',...box(r),name:identifier(e),
          shape:e.localName==='rect'?'rect':'ellipse',fill,line});
      } else if (e.localName==='line' && line.transparency<100) {
        const a=new DOMPoint(e.x1.baseVal.value,e.y1.baseVal.value).matrixTransform(matrix);
        const b=new DOMPoint(e.x2.baseVal.value,e.y2.baseVal.value).matrixTransform(matrix);
        items.push({kind:'shape',shape:'line',x:Math.min(a.x,b.x)-origin.x,y:Math.min(a.y,b.y)-origin.y,
          w:Math.abs(b.x-a.x),h:Math.abs(b.y-a.y),flipV:(b.y-a.y)*(b.x-a.x)<0,name:identifier(e),fill,line});
      }
      for (const child of e.childNodes) {
        if (child.nodeType===1 && !['title','desc'].includes(child.localName)) draw(child,alpha);
        else if (child.nodeType===3 && ['text','tspan'].includes(e.localName)) textNode(child,alpha*Number(s.fillOpacity),scale,s.fill);
      }
    }
    // The root opacity is already included by visit().
    draw(el,opacity/(Number(getComputedStyle(el).opacity)||1));
    return true;
  }
  function visit(el, parentOpacity=1) {
    const s=getComputedStyle(el), opacity=parentOpacity*Number(s.opacity);
    if (s.display==='none' || Number(s.opacity)===0 || ['SCRIPT','STYLE','TEMPLATE','NOSCRIPT'].includes(el.tagName)) return;
    const r=el.getBoundingClientRect();
    // A display:contents wrapper has no box but its children still render.
    if (s.display!=='contents' && !onSlide(r) && s.overflow!=='visible') return;
    const rendered=s.visibility==='visible';
    if (rendered) {
      if (el.tagName.toLowerCase()==='svg' && el.dataset.pptx!=='raster' && svg(el,opacity)) return;
      const pseudo=['::before','::after'].some(p=>{const v=getComputedStyle(el,p);return !['none','normal'].includes(v.content)&&v.display!=='none';});
      const transformed=s.transform!=='none' || (s.rotate && s.rotate!=='none') || (s.scale && s.scale!=='none');
      const clipped=el!==root && ['hidden','clip','scroll','auto'].some(v=>s.overflowX===v||s.overflowY===v)
        && (el.scrollWidth>el.clientWidth+1 || el.scrollHeight>el.clientHeight+1);
      const reason=el.dataset.pptx==='raster'?'Author requested a static image':
        ['svg','canvas','video','iframe','object','embed','input','select','textarea','button'].includes(el.tagName.toLowerCase()) ? `${el.tagName.toLowerCase()} is a static image` :
        s.backgroundImage!=='none'?'CSS background image or gradient':
        transformed?'CSS transform':s.filter!=='none'||s.backdropFilter!=='none'?'CSS filter':
        s.clipPath!=='none'||(s.maskImage && s.maskImage!=='none')?'CSS clip or mask':
        s.boxShadow!=='none'||s.textShadow!=='none'?'CSS shadow':
        s.mixBlendMode!=='normal'?'CSS blend mode':pseudo?'CSS generated content':
        s.writingMode!=='horizontal-tb'||s.direction==='rtl'?'Vertical or right-to-left text':
        clipped?'Clipped or scrollable content':
        s.display==='list-item'&&s.listStyleType!=='none'&&(!['disc','circle','square','decimal'].includes(s.listStyleType)||s.listStylePosition!=='outside')?'Custom list marker':null;
      if (reason) {bitmap(el,`${reason}; this element is a picture in PPTX and its internal text/shapes cannot be edited.`); return;}
      if (el.tagName==='IMG') {bitmap(el,null,true); return;}
      shape(el,s,opacity);
      if (s.display==='list-item' && s.listStyleType!=='none') {
        if (['disc','circle','square','decimal'].includes(s.listStyleType) && s.listStylePosition==='outside') {
          const siblings=[...el.parentElement.children].filter(e=>getComputedStyle(e).display==='list-item');
          const n=Number(el.getAttribute('value')) || (Number(el.parentElement.getAttribute('start'))||1)+siblings.indexOf(el);
          const marker=s.listStyleType==='decimal'?`${n}.`:({disc:'•',circle:'○',square:'▪'})[s.listStyleType];
          const size=parseFloat(s.fontSize), width=size*1.2;
          items.push({kind:'text',x:r.x-origin.x-width,y:r.y-origin.y+(parseFloat(s.paddingTop)||0),
            w:width-4,h:parseFloat(s.lineHeight)||size*1.2,name:`${identifier(el)} marker`,text:marker,
            style:{...textStyle(s,opacity),align:'right'}});
        }
      }
    }
    // Negative/positive z-index children are placed around ordinary content.
    const index=child=>child.nodeType===1 ? Number(getComputedStyle(child).zIndex)||0 : 0;
    const children=[...el.childNodes];
    children.sort((a,b)=>Math.sign(index(a))-Math.sign(index(b)) || index(a)-index(b));
    for (const child of children) {
      if (child.nodeType===1) visit(child,opacity);
      else if (child.nodeType===3 && rendered) textNode(child,opacity);
    }
  }
  visit(root);
  for (const item of items) item.name=`${id}/${item.name}/${++serial}`;
  return {id,width:origin.width,height:origin.height,items,warnings};
}

export async function captureSlide(page, slide) {
  const scene=await page.evaluate(extractScene,{id:slide.id});
  for (const item of scene.items) if (item.kind==='image') {
    const bytes=await page.screenshot({clip:item.clip,animations:'disabled',omitBackground:true,scale:'css'});
    item.data=`image/png;base64,${bytes.toString('base64')}`;
    delete item.clip;
  }
  return scene;
}

export async function writePptx(scenes, manifest, filename) {
  const pptx=new PptxGenJS();
  pptx.defineLayout({name:'PRESMITH',width:manifest.width/96,height:manifest.height/96});
  pptx.layout='PRESMITH'; pptx.title=manifest.title; pptx.subject='Presmith presentation';
  pptx.author=''; pptx.company=''; pptx.lang='en-US';
  const reports=[];
  for (let i=0;i<scenes.length;i++) {
    const scene=scenes[i], slide=pptx.addSlide();
    slide.name=manifest.slides[i].title || scene.id;
    const stats={slide_id:scene.id,text_boxes:0,shapes:0,images:0,rasterized_elements:0};
    for (const item of scene.items) {
      const pos={x:item.x/96,y:item.y/96,w:item.w/96,h:item.h/96,objectName:item.name,...(item.flipV?{flipV:true}:{})};
      if (item.kind==='text') {
        slide.addText(item.text,{...pos,w:pos.w+.06,...item.style,margin:0,paraSpaceAfter:0,paraSpaceBefore:0,
          breakLine:false,wrap:false,valign:'mid',isTextBox:true,fit:'shrink'});
        stats.text_boxes++;
      } else if (item.kind==='shape') {
        const line={...item.line}; delete line.style;
        if (['dashed','dotted'].includes(item.line.style)) line.dashType=item.line.style==='dotted'?'sysDot':'dash';
        slide.addShape(pptx.ShapeType[item.shape],{...pos,fill:item.fill,line,
          ...(item.shape==='roundRect'?{radius:item.radius/96,rectRadius:item.radius/96}:{})});
        stats.shapes++;
      } else {
        slide.addImage({...pos,data:item.data,altText:item.name});
        stats.images++;
        if (item.fallback) stats.rasterized_elements++;
      }
    }
    if (manifest.slides[i].notes) slide.addNotes(manifest.slides[i].notes);
    reports.push(stats);
  }
  const bytes=await pptx.write({outputType:'nodebuffer',compression:true});
  const zip=await JSZip.loadAsync(bytes,{checkCRC32:true});
  const slides=Object.keys(zip.files).filter(p=>/^ppt\/slides\/slide\d+\.xml$/.test(p));
  if (slides.length!==manifest.slides.length) throw new Error('PPTX slide count differs from manifest');
  await writeFile(filename,bytes);
  return {kind:'pptx',path:'deck.pptx',slides:slides.length,editable:true,editability:reports};
}
