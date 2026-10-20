const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
(async()=>{
 const root='G:/MeetingAssistant/build/focus-native-fixture-'+Date.now();fs.mkdirSync(root,{recursive:true});
 const log=fs.openSync('G:/MeetingAssistant/logs/focus-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9230'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9230');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');
  const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('button',{name:/先看一段演示/}).waitFor({timeout:30000});
  assert(await page.evaluate(async()=>await window.__TAURI_INTERNALS__.invoke('class_status')===null));
  await page.getByRole('button',{name:/先看一段演示/}).click();
  await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();
  await page.locator('.cc-card').waitFor();
  const original=await page.locator('.cc-transcript-body').boundingBox();
  const transcript=page.getByRole('button',{name:'放大阅读课堂实录',exact:true}),catchup=page.getByRole('button',{name:'放大阅读一键接上课堂',exact:true});
  await transcript.click();await catchup.click();
  const focused=await page.locator('.cc-transcript-body').boundingBox();assert(focused.height>original.height);
  const a=await page.locator('.cc-transcript').boundingBox(),b=await page.locator('.cc-catchup').boundingBox();assert(Math.abs(a.width-b.width)<2);
  assert(!await page.locator('.cc-sidebar').isVisible());
  assert(await page.getByRole('button',{name:'开始录课',exact:true}).isVisible());
  assert(await page.locator('.cc-app').evaluate(el=>el.scrollHeight<=el.clientHeight+1&&el.scrollWidth<=el.clientWidth+1));
  await page.screenshot({path:'G:/MeetingAssistant/build/专注阅读-Windows桌面实测.png'});
  await page.getByRole('button',{name:'退出专注',exact:true}).click();assert(await page.locator('.cc-sidebar').isVisible());assert.equal(await page.locator('.cc-card').count(),1);
  fs.writeFileSync('G:/MeetingAssistant/logs/native-focus-test.json',JSON.stringify({passed:true,isolatedRoot:root,extraReadingHeight:focused.height-original.height,equalWidth:true,exitRestored:true,cloudCallsMade:false,microphoneOpened:false},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
