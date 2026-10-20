const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'}),results=[];
 try{
  for(const [width,height,compact] of [[1280,720,false],[1600,960,false],[820,620,false],[560,720,true]]){
   const page=await browser.newPage({viewport:{width,height}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
   await page.addInitScript(()=>{
    window.isTauri=true;window.learningCalls=[];let id=0;const callbacks={},listeners={};
    const session={id:'lesson',title:'课程测试',status:'recording',elapsed_ms:1800000};
    const fact=text=>({text,segment_ids:['s1']});
    const card=(n,request={anchor_ms:900000+n*1000,lookback_ms:60000,mode:'catch_up'})=>({id:'card-'+n,session_id:'lesson',request,evidence:[],warning:'',card:{headline:fact('历史记录 '+n),key_points:[fact('原有课堂重点')],current_topic:null,tasks:[],explanation:null}});
    window.__TAURI_INTERNALS__={transformCallback:fn=>{callbacks[++id]=fn;return id;},invoke:async(cmd,args={})=>{
     window.learningCalls.push({cmd,args});
     if(cmd==='plugin:event|listen'){listeners[args.event]=args.handler;return args.handler;}
     if(cmd==='class_get_settings')return {settings:{processing_mode:'cloud',mode:'catch_up',lookback_ms:300000}};
     if(cmd==='class_history')return [session];if(cmd==='class_status')return session;
     if(cmd==='class_transcript')return [{id:'s1',start_ms:300000,end_ms:480000,text:'本节课讨论条件概率与贝叶斯定理。',is_final:true,revision:1}];
     if(cmd==='class_cards')return Array.from({length:12},(_,i)=>card(12-i));
     if(cmd==='get_audio_devices')return [];
     if(cmd==='class_catchup'){
      const result=card(Date.now(),args.request);
      if(args.request.mode==='ask'){result.card.headline=fact('针对你的问题');result.card.answer=fact('这是基于所选片段的连贯回答。'.repeat(35));result.card.key_points=[];}
      if(args.request.mode==='layered'){result.card.headline=fact('先看懂本节主线');result.card.layers=[{title:'理解条件概率',classroom:fact('课堂说明在已知条件下计算概率。'),explanation:'用白话解释条件概率的含义。',example:'假设我们只观察已经满足条件的一组样本。',relation:'先理解条件概率，再讨论贝叶斯定理。'}];}
      setTimeout(()=>callbacks[listeners['class-catchup']]?.({payload:{status:'completed',result}}),150);return result.id;
     }
     return null;
    }};
   });
   await page.goto('http://localhost:3118/'+(compact?'?compact=1':''),{waitUntil:'networkidle'});
   await page.getByRole('button',{name:'记录 12',exact:true}).waitFor();
   await page.getByRole('button',{name:'记录 12',exact:true}).click();assert.equal(await page.locator('.cc-card-history nav>button').count(),12);assert(!await page.locator('#cc-composer').isVisible());
   await page.getByRole('textbox',{name:'搜索补听记录',exact:true}).fill('历史记录 1');assert.equal(await page.locator('.cc-card-history nav>button').count(),4);
   await page.locator('.cc-card-history nav>button').filter({has:page.getByText('历史记录 1',{exact:true})}).click();
   await page.locator('.cc-card h3').filter({hasText:/^历史记录 1$/}).waitFor();assert.equal(await page.locator('.cc-card-history').count(),0);
   await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();
   await page.getByRole('button',{name:'记录 13',exact:true}).waitFor();
   assert.equal(await page.locator('#cc-composer').isVisible(),false);assert(await page.locator('.cc-tasks').isVisible());
   await page.locator('.cc-composer-toggle').click();await page.locator('.cc-mode-tabs').getByRole('button',{name:'提问',exact:true}).click();
   await page.getByRole('textbox',{name:'提问或总结要求',exact:true}).fill('请解释老师刚才的推理过程');
   await page.getByRole('button',{name:'指定起止时间',exact:true}).click();
   await page.getByRole('textbox',{name:'范围开始时间',exact:true}).fill('05:00');
   await page.getByRole('textbox',{name:'范围结束时间',exact:true}).fill('99:00');
   await page.getByRole('button',{name:'生成回答',exact:true}).click();await page.locator('.cc-error[role=alert]').waitFor();
   assert.equal(await page.evaluate(()=>window.learningCalls.filter(x=>x.cmd==='class_catchup').length),1);
   await page.getByRole('textbox',{name:'范围结束时间',exact:true}).fill('08:00');await page.getByRole('button',{name:'生成回答',exact:true}).click();
   await page.locator('.cc-answer p').waitFor();assert.equal(await page.locator('.cc-answer p').count(),1);assert.equal(await page.locator('.cc-point').count(),0);assert.equal(await page.locator('.cc-tasks').count(),0);
   const request=await page.evaluate(()=>window.learningCalls.filter(x=>x.cmd==='class_catchup').at(-1).args.request);
   assert.equal(request.anchor_ms,480000);assert.equal(request.lookback_ms,180000);assert.equal(request.mode,'ask');assert.equal(request.question,'请解释老师刚才的推理过程');
   await page.locator('.cc-composer-toggle').click();await page.locator('.cc-mode-tabs').getByRole('button',{name:'分层补听',exact:true}).click();
   await page.locator('.cc-run').click();await page.locator('.cc-layer summary').waitFor();
   assert(!await page.locator('.cc-layer-body').isVisible());const calls=await page.evaluate(()=>window.learningCalls.filter(x=>x.cmd==='class_catchup').length);
   await page.locator('.cc-layer summary').click();await page.getByText('AI辅助理解 · 不代表老师原话',{exact:true}).waitFor();
   assert.equal(await page.evaluate(()=>window.learningCalls.filter(x=>x.cmd==='class_catchup').length),calls);
   assert.equal(await page.getByRole('button',{name:'记录 15',exact:true}).count(),1);
   const snapshot=await page.evaluate(()=>{const root=document.querySelector('.cc-app'),res=document.querySelector('.cc-results');return {rootOverflow:root.scrollWidth>root.clientWidth+1||root.scrollHeight>root.clientHeight+1,resultHeight:res.clientHeight};});
   assert(!snapshot.rootOverflow);assert(snapshot.resultHeight>180);
   await page.screenshot({path:`G:/MeetingAssistant/build/分层补听-${width}x${height}.png`});
   await page.getByRole('button',{name:'就这个重点继续提问 →',exact:true}).click();
   assert((await page.getByRole('textbox',{name:'提问或总结要求',exact:true}).inputValue()).includes('理解条件概率'));
   assert.equal(await page.getByRole('textbox',{name:'范围开始时间',exact:true}).inputValue(),'05:00');
   await page.locator('.cc-composer-toggle').click();await page.getByRole('button',{name:'记录 15',exact:true}).click();
   await page.getByRole('textbox',{name:'搜索补听记录',exact:true}).fill('');assert.equal(await page.locator('.cc-card-history nav>button').count(),15);
   await page.screenshot({path:`G:/MeetingAssistant/build/补听历史菜单-${width}x${height}.png`});
   assert.deepEqual(errors,[]);results.push({width,height,compact,...snapshot,allHistoryRecords:15,request,expansionAddsAPICalls:false});await page.close();
  }
  fs.writeFileSync('G:/MeetingAssistant/logs/ui-learning-test.json',JSON.stringify({passed:true,results},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
