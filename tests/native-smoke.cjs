// Launch the app separately with WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS set to
// --remote-debugging-port=9224. This test never saves keys or makes cloud calls.
const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs');
(async()=>{
 let browser;
 for(let i=0;i<30;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9224');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
 if(!browser)throw Error('Desktop WebView debugging endpoint did not start');
 const context=browser.contexts()[0];let page=context.pages()[0];
 if(!page)page=await context.waitForEvent('page',{timeout:15000});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.getByRole('heading',{name:'走神一小会儿，也能接着听。'}).waitFor({timeout:30000});
 const snapshot=await page.evaluate(async()=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;
   const config=await invoke('class_get_settings');
   const history=await invoke('class_history');
   const devices=await invoke('get_audio_devices');
   const status=await invoke('class_status');
   let invalidRejected=false;
   try{await invoke('class_catchup',{sessionId:'test-no-session',request:{anchor_ms:60000,lookback_ms:60000,mode:'invalid',question:''}});}catch{invalidRejected=true;}
   const noAudio=await invoke('class_audio_file',{sessionId:'test-no-session'});
   return {data_dir:config.data_dir,shortcut_error:config.shortcut_error,
     settingsContainNoKeys:!('asr_key'in config.settings)&&!('llm_key'in config.settings),
     historyCount:history.length,deviceCount:devices.length,statusIsIdle:status===null,
     invalidRejected,missingSessionHasNoAudio:noAudio===null};
 });
 snapshot.pageUrl=page.url();
 if(/localhost:3118|127\.0\.0\.1:3118/.test(snapshot.pageUrl))throw Error('Desktop app still depends on the development/preview server');
 if(!snapshot.settingsContainNoKeys||!snapshot.invalidRejected||!snapshot.missingSessionHasNoAudio)throw Error('Native API checks failed');
 if(await page.getByText('浏览器预览 · 不会录音或调用API',{exact:true}).count())throw Error('Desktop app incorrectly detects browser preview');
 await page.getByRole('button',{name:'设置',exact:true}).click();
 await page.getByRole('dialog').waitFor();
 await page.getByRole('button',{name:'关闭设置'}).click();
 await page.screenshot({path:'G:/MeetingAssistant/build/课堂补听-Windows桌面实测.png',fullPage:true});
 fs.writeFileSync('G:/MeetingAssistant/logs/native-smoke.json',JSON.stringify({snapshot,pageErrors:errors,cloudCallsMade:false,keysModified:false},null,2));
 // Disconnect from the externally launched browser without closing its app.
 await browser.close();if(errors.length)process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1;});
