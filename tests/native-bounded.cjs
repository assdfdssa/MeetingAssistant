const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
(async()=>{
 const root='G:/MeetingAssistant/build/bounded-native-fixture-'+Date.now();fs.mkdirSync(root,{recursive:true});
 const log=fs.openSync('G:/MeetingAssistant/logs/bounded-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9228'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9228');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');
  const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('button',{name:/先看一段演示/}).waitFor({timeout:30000});
  const idle=await page.evaluate(async()=>await window.__TAURI_INTERNALS__.invoke('class_status')===null);assert(idle);
  await page.getByRole('button',{name:/先看一段演示/}).click();await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();await page.locator('.cc-card').waitFor();
  await page.locator('.cc-transcript-body').evaluate(el=>{for(let i=0;i<100;i++)el.appendChild(el.querySelector('.cc-segment').cloneNode(true));});
  await page.locator('.cc-card').evaluate(el=>{for(let i=0;i<25;i++)el.appendChild(el.querySelector('.cc-point').cloneNode(true));});
  await page.locator('.cc-results').hover();await page.mouse.wheel(0,20000);await page.waitForTimeout(200);
  const result=await page.evaluate(()=>{
   const app=document.querySelector('.cc-app'),res=document.querySelector('.cc-results'),body=document.querySelector('.cc-transcript-body');
   const bottom=res.querySelector('.cc-card').lastElementChild.getBoundingClientRect().bottom;
   const header=document.querySelector('.cc-recordbar').getBoundingClientRect();
   return {width:innerWidth,height:innerHeight,rootOverflow:app.scrollHeight>app.clientHeight+1,rootTop:app.scrollTop,transcriptCanScroll:body.scrollHeight>body.clientHeight,resultCanScroll:res.scrollHeight>res.clientHeight,resultAtBottom:bottom<=res.getBoundingClientRect().bottom+1,recordingControlsVisible:header.top>=0&&header.bottom<innerHeight,timeWhiteSpace:getComputedStyle(document.querySelector('.cc-segment>button')).whiteSpace,url:location.href};
  });
  assert(!result.rootOverflow);assert.equal(result.rootTop,0);assert(result.transcriptCanScroll&&result.resultCanScroll&&result.resultAtBottom&&result.recordingControlsVisible);assert.equal(result.timeWhiteSpace,'nowrap');
  await page.screenshot({path:'G:/MeetingAssistant/build/固定工作区-Windows桌面实测.png'});
  fs.writeFileSync('G:/MeetingAssistant/logs/native-bounded-test.json',JSON.stringify({isolatedRoot:root,result,cloudCallsMade:false,microphoneOpened:false},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
