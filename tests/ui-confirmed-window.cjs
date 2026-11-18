const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true,channel:'msedge'});const results=[];
 try{for(const partialOnly of [false,true]){
  const page=await browser.newPage({viewport:{width:1280,height:800}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(({partialOnly})=>{
   window.isTauri=true;window.windowCalls=[];let id=0;const callbacks={},listeners={};
   const session={id:'fixture',title:'时间范围测试',status:'recording',elapsed_ms:600000};
   window.emitFinal=()=>callbacks[listeners['class-transcript']]?.({payload:{session_id:'fixture',segment:{id:'new',start_ms:80000,end_ms:110000,text:'新确认的一句话',is_final:true,revision:2}}});
   window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks[++id]=fn;return id},invoke:async(cmd,args={})=>{
    window.windowCalls.push({cmd,args});if(cmd==='plugin:event|listen'){listeners[args.event]=args.handler;return args.handler;}
    if(cmd==='class_get_settings')return{settings:{processing_mode:'cloud',mode:'catch_up',lookback_ms:60000}};
    if(cmd==='class_status')return session;if(cmd==='class_history')return[session];
    if(cmd==='class_transcript')return [...(partialOnly?[]:[{id:'ready',start_ms:50000,end_ms:80000,text:'已确认的一句话',is_final:true,revision:1},{id:'blank',start_ms:580000,end_ms:590000,text:' ',is_final:true,revision:1}]),{id:'pending',start_ms:80000,end_ms:600000,text:'仍在识别',is_final:false,revision:1}];
    if(cmd==='get_audio_devices'||cmd==='class_cards')return[];
    if(cmd==='class_catchup'){const result={id:'card-'+Date.now(),session_id:'fixture',request:args.request,evidence:[],warning:'',card:{headline:{text:'测试补听结果',segment_ids:['ready']},key_points:[],current_topic:null,tasks:[],explanation:null}};setTimeout(()=>callbacks[listeners['class-catchup']]?.({payload:{status:'completed',result}}),50);return result.id;}
    return null;
   }};
  },{partialOnly});
  await page.goto('http://127.0.0.1:3118/',{waitUntil:'networkidle'});
  await page.getByText('仍在识别',{exact:false}).waitFor();
  await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();
  if(partialOnly){await page.getByText('还没有已确认的转写，请等一句话识别完成后再补听',{exact:true}).waitFor();assert.equal(await page.evaluate(()=>window.windowCalls.filter(c=>c.cmd==='class_catchup').length),0);}
  else{
   await page.getByRole('heading',{name:'测试补听结果',exact:true}).waitFor();
   let first=await page.evaluate(()=>window.windowCalls.filter(c=>c.cmd==='class_catchup')[0].args);
   assert.equal(first.request.anchor_ms,80000);assert.equal(first.request.lookback_ms,60000);assert.equal(first.endAtLastFinal,true);
   await page.evaluate(()=>window.emitFinal());await page.getByText('新确认的一句话',{exact:true}).waitFor();
   await page.locator('.cc-run').click();await page.waitForTimeout(120);
   let second=await page.evaluate(()=>window.windowCalls.filter(c=>c.cmd==='class_catchup')[1].args);assert.equal(second.request.anchor_ms,110000);assert.equal(first.request.anchor_ms,80000);
   await page.locator('.cc-composer-toggle').click();await page.locator('.cc-mode-tabs').getByRole('button',{name:'提问',exact:true}).click();
   await page.getByRole('button',{name:'指定起止时间',exact:true}).click();await page.getByRole('textbox',{name:'范围开始时间'}).fill('00:30');await page.getByRole('textbox',{name:'范围结束时间'}).fill('01:10');await page.getByRole('button',{name:'生成回答',exact:true}).click();await page.waitForTimeout(120);
   const explicit=await page.evaluate(()=>window.windowCalls.filter(c=>c.cmd==='class_catchup').at(-1).args);assert.equal(explicit.request.anchor_ms,70000);assert.equal(explicit.request.lookback_ms,40000);assert.equal(explicit.endAtLastFinal,false);
   results.push({first,second,explicit});
  }
  assert.deepEqual(errors,[]);results.push({partialOnly,passed:true});await page.close();
 }
 fs.writeFileSync('G:/MeetingAssistant/logs/ui-confirmed-window-test.json',JSON.stringify({passed:true,cloudCallsMade:false,microphoneOpened:false,results},null,2));console.log('Confirmed-window UI tests passed.');
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
