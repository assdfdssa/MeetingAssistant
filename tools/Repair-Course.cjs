// Explicit maintenance entry point; the normal product entry is the course menu.
// Requires --execute and a named course. Never starts recording or calls an API.
const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn}=require('node:child_process'),fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),crypto=require('node:crypto');
const args=process.argv.slice(2),value=k=>args[args.indexOf(k)+1];
const root=value('--root'),id=value('--session-id');
if(!args.includes('--execute')||!root||!id||!/^[a-zA-Z0-9-]+$/.test(id))throw Error('Specify --root, --session-id and --execute');
(async()=>{
 const job=Date.now(),profile=path.join(root,'build','repair-maintenance-profile-'+job),logPath=path.join(root,'logs','repair-maintenance-'+job+'.log');
 const log=fs.openSync(logPath,'w'),app=spawn(path.join(root,'build/native/debug/meetily.exe'),[],{cwd:root,env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:profile,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9234'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 const sha=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9234');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('button',{name:'设置',exact:true}).waitFor({timeout:30000});
  const before=await page.evaluate(async id=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;
   if(await invoke('class_status'))throw Error('Close the current recording before maintenance');
   const settings=await invoke('class_get_settings'),rows=[...await invoke('class_history',{archived:false}),...await invoke('class_history',{archived:true})];
   const lesson=rows.find(s=>s.id===id);if(!lesson)throw Error('Course not found');
   return {dataDir:settings.data_dir,lesson,cards:(await invoke('class_cards',{sessionId:id})).length,segments:(await invoke('class_transcript',{sessionId:id})).length};
  },id);
  assert.equal(path.resolve(before.dataDir),path.resolve(root,'data'));
  const candidates=fs.readdirSync(path.join(root,'data/recordings')).filter(name=>name.startsWith('Class-'+id+'_'));assert.equal(candidates.length,1);
  const folder=path.join(root,'data/recordings',candidates[0]),checkpoints=path.join(folder,'.checkpoints');
  const hashes=Object.fromEntries(fs.readdirSync(checkpoints).filter(n=>/^audio_chunk_\d+\.mp4$/.test(n)).map(name=>[name,sha(path.join(checkpoints,name))]));
  const transcriptHash=sha(path.join(folder,'transcripts.json'));
  const result=await page.evaluate(async id=>await window.__TAURI_INTERNALS__.invoke('class_repair_session',{sessionId:id}),id);
  for(const [name,hash] of Object.entries(hashes))assert.equal(sha(path.join(checkpoints,name)),hash);
  assert.equal(sha(path.join(folder,'transcripts.json')),transcriptHash);
  const after=await page.evaluate(async id=>{const invoke=window.__TAURI_INTERNALS__.invoke;return {cards:(await invoke('class_cards',{sessionId:id})).length,segments:(await invoke('class_transcript',{sessionId:id})).length,audio:await invoke('class_audio_file',{sessionId:id})};},id);
  assert.equal(after.cards,before.cards);assert(after.segments>=before.segments);assert(fs.existsSync(after.audio));assert(fs.existsSync(result.database_backup));
  await page.locator('.cc-history').filter({hasText:before.lesson.title}).first().click();
  await page.locator('audio').waitFor();await page.screenshot({path:path.join(root,'build','课程修复-研究方法.png')});
  const record={session_id:id,before:{status:before.lesson.status,cards:before.cards,segments:before.segments},after,originalCheckpointsUnchanged:Object.keys(hashes).length,originalTranscriptExportUnchanged:true,cloudCallsMade:false,microphoneOpened:false,...result};
  fs.writeFileSync(path.join(root,'logs','repair-'+id+'.json'),JSON.stringify(record,null,2));
  console.log(JSON.stringify(record));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
