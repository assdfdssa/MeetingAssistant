const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 try{
 const page=await browser.newPage({viewport:{width:1280,height:720}});
 await page.addInitScript(()=>{
  window.isTauri=true;const callbacks={},listeners={};let counter=0;
  const session={id:'live',title:'实时滚动验证',status:'recording',elapsed_ms:60000};
  const segment=i=>({id:'s'+i,start_ms:i*1000,end_ms:i*1000+900,text:'课堂转写测试内容 '+i,is_final:true,revision:1});
  window.emitTestSegment=i=>callbacks[listeners['class-transcript']]({payload:{session_id:'live',segment:segment(i)}});
  window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks[++counter]=fn;return counter;},invoke:async(cmd,args={})=>{
   if(cmd==='plugin:event|listen'){listeners[args.event]=args.handler;return args.handler;}
   if(cmd==='class_get_settings')return {settings:{asr_model:'test',llm_model:'test',llm_base_url:'https://example.com',vocabulary:'',mode:'catch_up',lookback_ms:300000}};
   if(cmd==='class_history')return [session];if(cmd==='class_status')return session;
   if(cmd==='class_transcript')return Array.from({length:80},(_,i)=>segment(i));
   if(cmd==='get_audio_devices'||cmd==='class_cards')return [];return null;
  }};
 });
 await page.goto('http://localhost:3118',{waitUntil:'networkidle'});await page.locator('.cc-segment').nth(79).waitFor();
 const body=page.locator('.cc-transcript-body');await page.waitForTimeout(200);
 assert(await body.evaluate(el=>el.scrollHeight-el.scrollTop-el.clientHeight<3));
 await body.hover();await page.mouse.wheel(0,-1500);await page.getByRole('button',{name:'回到最新 ↓',exact:true}).waitFor();
 const top=await body.evaluate(el=>el.scrollTop);await page.evaluate(()=>window.emitTestSegment(80));await page.waitForTimeout(200);
 assert(Math.abs((await body.evaluate(el=>el.scrollTop))-top)<2,'New text must preserve reading position');
 await page.getByRole('button',{name:'回到最新 ↓',exact:true}).click();await page.evaluate(()=>window.emitTestSegment(81));await page.waitForTimeout(200);
 assert(await body.evaluate(el=>el.scrollHeight-el.scrollTop-el.clientHeight<3));
 assert.equal(await page.locator('.cc-app').evaluate(el=>el.scrollTop),0);
 fs.writeFileSync('G:/MeetingAssistant/logs/ui-follow-test.json',JSON.stringify({checks:['initial latest position','scrolling up suspends following','new text preserves position','return to latest resumes following','fixed window'],passed:true},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
