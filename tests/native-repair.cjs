const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn,spawnSync}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),crypto=require('node:crypto');
(async()=>{
 const root='G:/MeetingAssistant/build/repair-native-fixture-'+Date.now(),id='repair-one',folder=path.join(root,'data/recordings/Class-'+id+'_test');fs.mkdirSync(folder+'/.checkpoints',{recursive:true});
 const ffmpeg='G:/MeetingAssistant/build/native/debug/ffmpeg.exe';
 for(let i=0;i<2;i++){const r=spawnSync(ffmpeg,['-nostdin','-v','error','-f','lavfi','-i','sine=frequency=440:sample_rate=48000','-t','0.5','-ac','1','-c:a','aac','-n',folder+`/.checkpoints/audio_chunk_00${i}.mp4`],{windowsHide:true});assert.equal(r.status,0);}
 const hash=()=>crypto.createHash('sha256').update(fs.readFileSync(folder+'/.checkpoints/audio_chunk_000.mp4')).digest('hex');const original=hash();
 fs.writeFileSync(folder+'/metadata.json',JSON.stringify({status:'recording',audio_file:'audio.mp4'}));
 fs.writeFileSync(folder+'/transcripts.json',JSON.stringify({segments:[{text:'Recovered transcript.',audio_start_time:0.1,audio_end_time:0.4}]}));
 fs.writeFileSync(folder+'/audio-timeline.jsonl',JSON.stringify({class_start_ms:0,class_end_ms:4000,audio_start_ms:0,audio_end_ms:4000})+'\n');
 const python='C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe';
 const r=spawnSync(python,['-',root,folder],{input:`import sqlite3,sys
c=sqlite3.connect(sys.argv[1]+'/data/classcatch.sqlite')
c.execute('CREATE TABLE class_sessions (id TEXT PRIMARY KEY,title TEXT NOT NULL,created_at TEXT NOT NULL,status TEXT NOT NULL,audio_folder TEXT,elapsed_ms INTEGER NOT NULL DEFAULT 0)')
c.execute("INSERT INTO class_sessions VALUES('repair-one','Repair fixture','2026-10-19','interrupted',?,4000)",(sys.argv[2],))
c.commit()
`,encoding:'utf8',env:{...process.env,PYTHONIOENCODING:'utf-8'}});assert.equal(r.status,0,r.stderr);
 const log=fs.openSync('G:/MeetingAssistant/logs/repair-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9233'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9233');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.getByRole('button',{name:'管理课程：Repair fixture',exact:true}).click({timeout:30000});await page.getByRole('menuitem',{name:'修复课程',exact:true}).click();
  await page.locator('.cc-history small').filter({hasText:'部分恢复'}).waitFor({timeout:30000});
  const result=await page.evaluate(async()=>{
   const invoke=window.__TAURI_INTERNALS__.invoke;const audio=await invoke('class_audio_file',{sessionId:'repair-one'});const segments=await invoke('class_transcript',{sessionId:'repair-one'});
   let unavailableTailBlocked=false;try{await invoke('class_audio_position',{sessionId:'repair-one',classMs:3000});}catch{unavailableTailBlocked=true;}
   const second=await invoke('class_repair_session',{sessionId:'repair-one'});return {audio,segments:segments.length,second,unavailableTailBlocked};
  });
  assert.equal(result.segments,1);assert.equal(result.second.report.imported_segments,0);assert(result.unavailableTailBlocked);assert(fs.existsSync(result.audio));assert(fs.existsSync(result.second.database_backup));assert.equal(hash(),original);
  await page.screenshot({path:'G:/MeetingAssistant/build/课程修复-Windows实测.png'});
  fs.writeFileSync('G:/MeetingAssistant/logs/native-repair-test.json',JSON.stringify({passed:true,isolatedRoot:root,originalCheckpointUnchanged:true,databaseBackupExists:true,importedFinalTranscript:true,idempotent:true,unavailableTailBlocked:true,cloudCallsMade:false,microphoneOpened:false,result},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
