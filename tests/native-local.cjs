const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
(async()=>{
 const root='G:/MeetingAssistant/build/local-native-fixture-'+Date.now();fs.mkdirSync(root+'/data',{recursive:true});
 fs.copyFileSync('G:/MeetingAssistant/data/local-models.json',root+'/data/local-models.json');
 const log=fs.openSync('G:/MeetingAssistant/logs/local-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9231'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9231');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');
  const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('button',{name:'设置',exact:true}).waitFor({timeout:30000});
  const result=await page.evaluate(async()=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;
   const initial=await invoke('class_get_settings');
   const status=await invoke('class_local_status');
   const tested=await invoke('class_test_local');
   const afterTest=await invoke('class_get_settings');
   const settings={...initial.settings,processing_mode:'local'};
   let rejectedWithoutAck=false;try{await invoke('class_save_settings',{settings,asrKey:null,llmKey:null,localAcknowledged:false});}catch{rejectedWithoutAck=true;}
   await invoke('class_save_settings',{settings,asrKey:null,llmKey:null,localAcknowledged:true});
   const saved=await invoke('class_get_settings');
   let cloudTestBlocked=false;try{await invoke('class_test_provider',{kind:'llm'});}catch(e){cloudTestBlocked=String(e).includes('本地');}
   await invoke('class_save_settings',{settings:initial.settings,asrKey:null,llmKey:null,localAcknowledged:false});
   return {status,tested,initialMode:initial.settings.processing_mode,afterTestMode:afterTest.settings.processing_mode,savedMode:saved.settings.processing_mode,rejectedWithoutAck,cloudTestBlocked};
  });
  assert(result.status.files_ready);assert(result.status.asr_directory.includes('qwen3-asr-1.7b-openvino'));assert(result.status.runtime_directory.startsWith('G:'));assert(!result.status.language_notice.includes('不支持中文'));assert(result.tested.tested);assert.equal(result.initialMode,'cloud');assert.equal(result.afterTestMode,'cloud');assert.equal(result.savedMode,'local');assert(result.rejectedWithoutAck);assert(result.cloudTestBlocked);
  await page.getByRole('button',{name:'设置',exact:true}).click();await page.getByText('00 / 处理方式',{exact:true}).waitFor();
  await page.screenshot({path:'G:/MeetingAssistant/build/本地应急-设置实测.png'});
  await page.getByText('纯本地应急（中英混合）',{exact:true}).waitFor();
  fs.writeFileSync('G:/MeetingAssistant/logs/native-local-test.json',JSON.stringify({passed:true,isolatedRoot:root,...result,cloudCallsMade:false,microphoneOpened:false,asrTest:'OpenVINO 1.7B runtime smoke test on silence; recognition examples measured separately in openvino-asr reports'},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
