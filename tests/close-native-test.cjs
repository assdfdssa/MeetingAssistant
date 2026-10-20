const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
(async()=>{
 const browser=await chromium.connectOverCDP('http://127.0.0.1:9224');
 const page=browser.contexts()[0].pages()[0];
 const idle=await page.evaluate(async()=>await window.__TAURI_INTERNALS__.invoke('class_status')===null);
 if(!idle)throw Error('A recording is active; leave the desktop app running');
 try{await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('plugin:process|exit',{code:0}));}catch(e){if(!String(e).includes('closed'))throw e;}
 await browser.close().catch(()=>{});
})().catch(e=>{console.error(e);process.exitCode=1;});
