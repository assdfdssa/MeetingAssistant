const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{const browser=await chromium.launch({headless:true,channel:'msedge'});const results=[];
try {for(const [width,height] of [[1280,720],[820,620],[560,720]]){
 const page=await browser.newPage({viewport:{width,height}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.isTauri=true;window.promptCalls=[];let callback=0,revision=0;
  const titles=['补听重点','提问','分层补听','云端摘要连接测试','本地摘要检测·规则','本地摘要检测·请求','语音识别·课程术语'];
  let items=['catch_up','ask','layered','connection_test','local_test_system','local_test_user','asr_context'].map((id,i)=>({id,title:titles[i],description:'此项用于测试提示词编辑行为。',text:i===6?'':'默认规则 '+id,default_text:i===6?'':'默认规则 '+id,max_length:i>=3&&i<6?500:20000,allow_empty:i===6}));
  window.__TAURI_INTERNALS__={transformCallback:()=>++callback,invoke:async(cmd,args={})=>{
   window.promptCalls.push({cmd,args});
   if(cmd==='class_get_settings')return {settings:{processing_mode:'cloud',mode:'catch_up',lookback_ms:300000,prompt_overrides:{}}};
   if(cmd==='class_get_prompts')return {items:JSON.parse(JSON.stringify(items)),revision:String(revision)};
   if(cmd==='class_save_prompts'){
    if(window.failPromptSave){window.failPromptSave=false;throw new Error('模拟保存失败，原配置保留');}
    items=items.map(item=>({...item,text:args.texts[item.id]}));revision++;
    return {vocabulary:args.texts.asr_context,prompt_overrides:args.texts};
   }
   if(cmd==='class_status')return null;
   if(cmd==='class_history'||cmd==='get_audio_devices')return [];
   return null;
  }};
 });
 await page.goto('http://localhost:3118/',{waitUntil:'networkidle',timeout:180000});
 const open=page.getByRole('button',{name:'提示词设置',exact:true});await open.click();
 const modal=page.getByRole('dialog',{name:'提示词设置',exact:true});
 try{await modal.getByRole('button',{name:/^提问/}).waitFor();}catch(e){console.error(await page.locator('body').innerText());console.error(await page.evaluate(()=>window.promptCalls));console.error(errors);await page.screenshot({path:'G:/MeetingAssistant/build/提示词设置-test-error.png'});throw e;}
 assert.equal(await modal.locator('nav button').count(),7);
 await modal.getByRole('button',{name:/^提问/}).click();const editor=modal.getByRole('textbox',{name:'提示词原文',exact:true});
 await editor.fill('请详细解释课堂内容，返回约定JSON。');
 await modal.getByRole('button',{name:'关闭提示词设置',exact:true}).click();await modal.getByText('修改尚未保存，是否放弃？').waitFor();
 await modal.getByRole('button',{name:'继续编辑',exact:true}).click();
 await page.evaluate(()=>window.failPromptSave=true);await modal.getByRole('button',{name:'保存提示词',exact:true}).click();await modal.getByRole('alert').waitFor();
 assert.equal(await editor.inputValue(),'请详细解释课堂内容，返回约定JSON。');
 await modal.getByRole('button',{name:'保存提示词',exact:true}).click();await modal.getByText(/已保存。生成提示词/).waitFor();
 await modal.getByRole('button',{name:'关闭提示词设置',exact:true}).click();await open.click();await modal.getByRole('button',{name:/^提问/}).click();
 assert.equal(await editor.inputValue(),'请详细解释课堂内容，返回约定JSON。');
 await modal.getByRole('button',{name:'本项恢复默认',exact:true}).click();assert.equal(await editor.inputValue(),'默认规则 ask');
 await modal.getByRole('button',{name:'保存提示词',exact:true}).click();await modal.getByText(/已保存。生成提示词/).waitFor();
 await modal.getByRole('button',{name:/^语音识别/}).click();await editor.fill('Bayes theorem、后验概率');await modal.getByRole('button',{name:'保存提示词',exact:true}).click();await modal.getByText(/已保存。生成提示词/).waitFor();
 await page.screenshot({path:`G:/MeetingAssistant/build/提示词设置-${width}.png`,fullPage:true});
 assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
 const bounds=await modal.boundingBox();assert(bounds.x>=0&&bounds.width<=width);
 const calls=await page.evaluate(()=>window.promptCalls);assert(!calls.some(c=>['class_catchup','class_start','class_test_provider'].includes(c.cmd)));
 assert.equal(calls.filter(c=>c.cmd==='class_save_prompts').at(-1).args.texts.ask,'默认规则 ask');assert.equal(errors.length,0,errors.join('\n'));
 results.push({width,height,passed:true,checks:['7 templates','edit/save/reopen','failed save keeps draft','restore default','ASR context','discard confirmation','bounded layout','no API call'],pageErrors:errors});await page.close();
}}finally{await browser.close();fs.writeFileSync('G:/MeetingAssistant/logs/ui-prompt-settings.json',JSON.stringify(results,null,2));}})().catch(e=>{console.error(e);process.exitCode=1;});
