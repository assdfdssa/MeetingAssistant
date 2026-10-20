const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 try {
 const page=await browser.newPage({viewport:{width:1280,height:720}});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(()=>{
  window.isTauri=true;window.managementCalls=[];window.folderCalls=[];
  const rows=[{id:'test-one',title:'管理测试课程',status:'completed',created_at:'2026-10-16',elapsed_ms:60000,archived:false},{id:'test-live',title:'进行中课程',status:'recording',elapsed_ms:1000,archived:false}];
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args={})=>{
   if(command==='class_get_settings')return {settings:{asr_model:'test',llm_model:'test',llm_base_url:'https://example.com',vocabulary:'',mode:'catch_up',lookback_ms:300000}};
   if(command==='class_history')return rows.filter(r=>r.archived===!!args.archived);
   if(command==='class_status')return rows.find(r=>r.id==='test-live');
   if(command==='get_audio_devices'||command==='class_transcript'||command==='class_cards')return [];
   if(command==='class_audio_file')return null;
   if(command==='class_rename_session'){const row=rows.find(r=>r.id===args.sessionId);row.title=args.title.trim();return row.title;}
   if(command==='class_open_folder'){window.folderCalls.push(args.sessionId);return null;}
   if(command==='class_manage_session'){
    window.managementCalls.push(args);
    const i=rows.findIndex(r=>r.id===args.sessionId);
    if(args.action==='delete')rows.splice(i,1);else rows[i].archived=args.action==='archive';
    return {warning:null};
   }
   return 1;
  }};
 });
 await page.goto('http://localhost:3118',{waitUntil:'networkidle'});
 let manage=page.getByRole('button',{name:'管理课程：管理测试课程',exact:true});
 await manage.waitFor();await page.waitForTimeout(700);
 await page.getByRole('button',{name:'管理课程：进行中课程',exact:true}).click();
 assert(await page.getByRole('menuitem',{name:'重命名',exact:true}).isEnabled());assert(await page.getByRole('menuitem',{name:'在资源管理器中打开',exact:true}).isEnabled());
 assert(!(await page.getByRole('menuitem',{name:'彻底删除',exact:true}).isEnabled()));assert(!(await page.getByRole('menuitem',{name:'仅归档',exact:true}).isEnabled()));
 await page.waitForFunction(()=>document.activeElement?.getAttribute('role')==='menuitem');await page.keyboard.press('Escape');
 await manage.click();await page.getByRole('menuitem',{name:'重命名',exact:true}).click();
 const nameInput=page.getByRole('textbox',{name:'新的课程名称',exact:true});await nameInput.fill('');assert(!(await page.getByRole('button',{name:'保存名称',exact:true}).isEnabled()));
 await nameInput.fill('英语课堂');await nameInput.press('Enter');await page.getByRole('dialog',{name:'重命名课程',exact:true}).waitFor({state:'hidden'});
 manage=page.getByRole('button',{name:'管理课程：英语课堂',exact:true});await manage.click();
 const menu=page.getByRole('menu',{name:'课程设置',exact:true});await menu.waitFor();
 const box=await menu.boundingBox();assert(box.y>=0&&box.y+box.height<=720);
 await page.screenshot({path:'G:/MeetingAssistant/build/课程设置-操作菜单.png'});
 await page.getByRole('menuitem',{name:'在资源管理器中打开',exact:true}).click();assert.deepEqual(await page.evaluate(()=>window.folderCalls),['test-one']);
 await manage.click();await page.waitForFunction(()=>document.activeElement?.getAttribute('role')==='menuitem');await page.keyboard.press('Escape');await menu.waitFor({state:'hidden'});
 await manage.click();await page.getByRole('menuitem',{name:'仅归档',exact:true}).click();let dialog=page.getByRole('dialog',{name:'管理课程',exact:true});
 await dialog.getByRole('button',{name:'仅归档',exact:true}).click();await dialog.waitFor({state:'hidden'});
 assert.equal(await manage.count(),0);
 await page.getByRole('button',{name:'已归档',exact:true}).click();await manage.click();
 await page.getByRole('menuitem',{name:'恢复课程',exact:true}).click();
 await page.getByRole('button',{name:'恢复课程',exact:true}).click();
 await page.getByRole('button',{name:'返回记录',exact:true}).click();await manage.click();
 await page.getByRole('menuitem',{name:'彻底删除',exact:true}).click();
 await page.getByRole('radio').nth(1).check();
 assert.equal(await page.getByRole('button',{name:'永久删除',exact:true}).isEnabled(),false);
 await page.screenshot({path:'G:/MeetingAssistant/build/课程管理-删除确认.png'});
 await page.getByRole('checkbox',{name:'我确认永久删除这节课程及其全部文件'}).check();
 await page.getByRole('button',{name:'永久删除',exact:true}).click();await dialog.waitFor({state:'hidden'});
 assert.equal(await manage.count(),0);
 const calls=await page.evaluate(()=>window.managementCalls);
 assert.deepEqual(calls.map(c=>c.action),['archive','restore','delete']);assert.deepEqual(errors,[]);
 fs.writeFileSync('G:/MeetingAssistant/logs/ui-history-test.json',JSON.stringify({mockBackend:true,checks:['course settings menu','active course rename and browse enabled','active course deletion protected','rename','empty name rejected','folder action','Escape closes menu','menu fits viewport','archive','restore','delete confirmation'],calls,errors},null,2));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
