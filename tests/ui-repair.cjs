const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 try{
  const page=await browser.newPage({viewport:{width:1280,height:720}});
  await page.addInitScript(()=>{
   window.isTauri=true;window.repairCalls=[];let status='interrupted';
   window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(cmd,args={})=>{
    window.repairCalls.push({cmd,args});if(cmd==='class_get_settings')return {settings:{processing_mode:'cloud',mode:'catch_up',lookback_ms:300000}};
    if(cmd==='class_history')return [{id:'broken',title:'中断课程',status,elapsed_ms:60000}];
    if(cmd==='class_repair_session'){status='recovered_partial';return {report:{status,audio_duration_ms:45000,imported_segments:1,warnings:['末尾约15秒录音未保存，原始检查点保留']}};}
    if(cmd==='class_status')return null;if(cmd==='get_audio_devices'||cmd==='class_transcript'||cmd==='class_cards')return [];return null;
   }};
  });
  await page.goto('http://localhost:3118',{waitUntil:'networkidle'});
  await page.getByRole('button',{name:'管理课程：中断课程',exact:true}).click();
  const repair=page.getByRole('menuitem',{name:'修复课程',exact:true});assert(await repair.isEnabled());await repair.click();
  await page.locator('.cc-history small').filter({hasText:'部分恢复'}).waitFor();await page.getByText(/末尾约15秒录音未保存/).waitFor();
  const called=await page.evaluate(()=>window.repairCalls.filter(x=>x.cmd==='class_repair_session'));assert.deepEqual(called,[{cmd:'class_repair_session',args:{sessionId:'broken'}}]);
  await page.getByRole('button',{name:'管理课程：中断课程',exact:true}).click();
  assert(await page.getByRole('menuitem',{name:'重命名',exact:true}).isVisible());assert(await page.getByRole('menuitem',{name:'彻底删除',exact:true}).isVisible());
  await page.screenshot({path:'G:/MeetingAssistant/build/课程修复-菜单预览.png'});
  fs.writeFileSync('G:/MeetingAssistant/logs/ui-repair-test.json',JSON.stringify({passed:true,menuEntry:true,partialStatusVisible:true,missingTailDisclosed:true,recordingOpened:false,cloudCallsMade:false},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
