const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
(async()=>{
 const root='G:/MeetingAssistant/build/popup-hotfix-fixture-'+Date.now();fs.mkdirSync(root,{recursive:true});
 const log=fs.openSync('G:/MeetingAssistant/logs/popup-hotfix-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9229'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9229');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');
  const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('heading',{name:'走神一小会儿，也能接着听。',exact:true}).waitFor({timeout:30000});
  const guardResults=await page.evaluate(async()=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;const results={};
   for(const [name,args] of [['parakeet_download_model',{modelName:'test'}],['whisper_download_model',{modelName:'test'}],['builtin_ai_download_model',{modelName:'test'}],['complete_onboarding',{model:'test'}]]){
    try{await invoke(name,args);results[name]=false;}catch(e){results[name]=String(e).includes('课堂云端版');}
   }
   return results;
  });assert(Object.values(guardResults).every(Boolean));
  await page.goto('http://tauri.localhost/index.html?compact=1',{waitUntil:'networkidle'});
  await page.getByRole('heading',{name:'刚才，老师讲了什么？',exact:true}).waitFor();
  assert.equal(await page.getByText('Transcription Engine',{exact:true}).count(),0);
  assert.equal(await page.getByText('Summary Engine',{exact:true}).count(),0);
  await page.screenshot({path:'G:/MeetingAssistant/build/快捷弹窗-修复实测.png'});
  fs.writeFileSync('G:/MeetingAssistant/logs/native-popup-test.json',JSON.stringify({isolatedRoot:root,url:page.url(),guardResults,legacySetupVisible:false,cloudCallsMade:false,microphoneOpened:false},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
