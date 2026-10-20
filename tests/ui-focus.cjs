const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'}),results=[];
 try{
  for(const [width,height] of [[1280,720],[1600,960],[820,620]]){
   const page=await browser.newPage({viewport:{width,height}});
   await page.addInitScript(()=>{
    window.isTauri=true;window.focusCommands=[];
    const session={id:'focus-test',title:'专注布局验证',status:'recording',elapsed_ms:120000};
    window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(cmd,args={})=>{
     window.focusCommands.push(cmd);
     if(cmd==='class_get_settings')return {settings:{processing_mode:'cloud',mode:'catch_up',lookback_ms:300000}};
     if(cmd==='class_history')return [session];if(cmd==='class_status')return session;
     if(cmd==='class_transcript')return Array.from({length:100},(_,i)=>({id:'s'+i,start_ms:i*1000,end_ms:i*1000+900,text:'This is a classroom transcript. 这是用于验证阅读空间的课堂转写内容。',is_final:true,revision:1}));
     if(cmd==='class_cards')return [{id:'c1',session_id:session.id,request:{anchor_ms:120000,lookback_ms:60000,mode:'catch_up'},card:{headline:{text:'课堂摘要',segment_ids:[]},key_points:Array.from({length:30},()=>({text:'这是课堂中的一个重点，测试长摘要独立滚动。',segment_ids:[]})),current_topic:null,tasks:[],explanation:null},evidence:[],warning:''}];
     if(cmd==='get_audio_devices')return [];return null;
    }};
   });
   await page.goto('http://localhost:3118',{waitUntil:'networkidle'});
   await page.locator('.cc-segment').nth(99).waitFor();
   await page.getByRole('button',{name:'提问',exact:true}).click();
   await page.locator('.cc-question').fill('什么是条件概率？');
   const original=await page.locator('.cc-transcript-body').boundingBox();
   const transcript=page.getByRole('button',{name:'放大阅读课堂实录',exact:true});
   const catchup=page.getByRole('button',{name:'放大阅读一键接上课堂',exact:true});
   await transcript.click();
   assert.equal(await transcript.getAttribute('aria-pressed'),'true');
   assert(!await page.locator('.cc-sidebar').isVisible());
   assert(!await page.locator('.cc-topbar').isVisible());
   assert(await page.getByRole('button',{name:'结束并保存',exact:true}).isVisible());
   assert(await page.getByRole('button',{name:'暂停',exact:true}).isVisible());
   const enlarged=await page.locator('.cc-transcript-body').boundingBox();
   assert(enlarged.height>original.height+40,'Focus must increase reading height');
   if(width>900){const a=await page.locator('.cc-transcript').boundingBox(),b=await page.locator('.cc-catchup').boundingBox();assert(a.width/b.width>1.7);}
   await catchup.click();
   const equalA=await page.locator('.cc-transcript').boundingBox(),equalB=await page.locator('.cc-catchup').boundingBox();
   assert(Math.abs(equalA.width-equalB.width)<2,'Both expanded panels split space equally');
   await page.screenshot({path:`G:/MeetingAssistant/build/专注阅读-双栏-${width}x${height}.png`});
   await transcript.click();
   assert.equal(await transcript.getAttribute('aria-pressed'),'false');
   assert.equal(await catchup.getAttribute('aria-pressed'),'true');
   if(width>900){const a=await page.locator('.cc-transcript').boundingBox(),b=await page.locator('.cc-catchup').boundingBox();assert(b.width/a.width>1.7);}
   const body=page.locator('.cc-transcript-body'),res=page.locator('.cc-results');
   await body.evaluate(el=>el.scrollTop=100);const top=await body.evaluate(el=>el.scrollTop);
   await res.hover();await page.mouse.wheel(0,1200);await page.waitForTimeout(150);
   assert(await res.evaluate(el=>el.scrollTop>0));assert.equal(await body.evaluate(el=>el.scrollTop),top);
   assert(await page.locator('.cc-question').inputValue()==='什么是条件概率？');
   const overflow=await page.locator('.cc-app').evaluate(el=>el.scrollWidth>el.clientWidth+1||el.scrollHeight>el.clientHeight+1);
   assert(!overflow,'Focus must stay inside viewport');
   assert(await res.evaluate(el=>el.clientHeight>80),'Summary must retain usable reading area');
   await page.screenshot({path:`G:/MeetingAssistant/build/专注阅读-摘要优先-${width}x${height}.png`});
   await page.getByRole('button',{name:'退出专注',exact:true}).click();
   assert(!await page.locator('.cc-app').evaluate(el=>el.classList.contains('cc-focused')));
   assert.equal(await catchup.getAttribute('aria-pressed'),'false');
   assert(await page.locator('.cc-sidebar').isVisible());
   assert.equal(await page.locator('.cc-segment').count(),100);
   await transcript.focus();await page.keyboard.press('Space');assert.equal(await transcript.getAttribute('aria-pressed'),'true');
   await transcript.press('Enter');assert.equal(await transcript.getAttribute('aria-pressed'),'false');
   const mutations=await page.evaluate(()=>window.focusCommands.filter(cmd=>['class_start','class_stop','class_pause','class_catchup','class_save_settings'].includes(cmd)));
   assert.deepEqual(mutations,[],'Changing layout must never alter recording or settings');
   results.push({width,height,extraReadingHeight:Math.round(enlarged.height-original.height),independentScroll:true,preservedInput:true,recordingCommandsIssued:0});
   await page.close();
  }
  fs.writeFileSync('G:/MeetingAssistant/logs/ui-focus-test.json',JSON.stringify({passed:true,results},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
