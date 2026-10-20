const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'}),results=[];
 try{
  for(const route of ['/', '/index.html', '/?compact=1', '/index.html?compact=1', '/settings.html']){
   const page=await browser.newPage({viewport:{width:900,height:720}});
   await page.addInitScript(()=>{
    window.isTauri=true;window.testCommands=[];
    window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(cmd,args)=>{
     window.testCommands.push(cmd);
     if(cmd==='class_get_settings')return {settings:{asr_model:'qwen3-asr-flash-realtime',llm_model:'qwen-plus',llm_base_url:'https://dashscope.aliyuncs.com/compatible-mode/v1',vocabulary:'',mode:'catch_up',lookback_ms:300000}};
     if(cmd==='get_audio_devices'||cmd==='class_history'||cmd==='class_transcript'||cmd==='class_cards')return [];
     if(cmd==='class_status')return null;
     if(cmd==='get_onboarding_status'||cmd.includes('download')||cmd==='complete_onboarding')throw Error('Legacy setup must never run in the cloud classroom');
     return 1;
    }};
   });
   await page.goto('http://localhost:3118'+route,{waitUntil:'networkidle'});
   const compact=route.includes('compact=1');await page.getByRole('heading',{name:compact?'刚才，老师讲了什么？':'走神一小会儿，也能接着听。',exact:true}).waitFor();
   const commands=await page.evaluate(()=>window.testCommands);
   assert(!commands.some(c=>c==='get_onboarding_status'||c.includes('download')||c==='complete_onboarding'));
   results.push({route,compact,legacySetupInvoked:false});await page.close();
  }
  fs.writeFileSync('G:/MeetingAssistant/logs/popup-route-test.json',JSON.stringify({results},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
