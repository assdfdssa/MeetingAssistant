// Exercise the real desktop commands against an isolated, disposable course.
// Never opens a microphone, sends cloud requests, or edits the user's courses.
const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn,spawnSync}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const python='C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe';
const root='G:/MeetingAssistant/build/history-native-fixture-'+Date.now();
const id='native-history-one';
const folder=path.join(root,'data','recordings',`Class-${id}_test`);
function sql(script){const r=spawnSync(python,['-',path.join(root,'data','classcatch.sqlite')],{input:script,encoding:'utf8',env:{...process.env,PYTHONIOENCODING:'utf-8'}});if(r.status!==0)throw Error(r.stderr);return r.stdout.trim();}
(async()=>{
 if(fs.existsSync(root))throw Error('Fixture directory already exists; inspect it before another run');
 fs.mkdirSync(folder,{recursive:true});fs.writeFileSync(path.join(folder,'audio.mp4'),'test audio');fs.writeFileSync(path.join(folder,'transcripts.json'),'test words');
 sql(`import sqlite3,sys,json
c=sqlite3.connect(sys.argv[1])
c.execute('CREATE TABLE class_sessions (id TEXT PRIMARY KEY,title TEXT NOT NULL,created_at TEXT NOT NULL,status TEXT NOT NULL,audio_folder TEXT,elapsed_ms INTEGER NOT NULL DEFAULT 0)')
c.execute('CREATE TABLE class_segments (session_id TEXT NOT NULL,id TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(session_id,id))')
c.execute('CREATE TABLE class_events (seq INTEGER PRIMARY KEY,session_id TEXT NOT NULL,payload TEXT NOT NULL)')
c.execute('CREATE TABLE class_cards (id TEXT PRIMARY KEY,session_id TEXT NOT NULL,payload TEXT NOT NULL)')
c.execute("INSERT INTO class_sessions VALUES(?,?,'2026-10-16','completed',?,60000)",(${JSON.stringify(id)},'原生管理验证',${JSON.stringify(folder)}))
c.execute("INSERT INTO class_segments VALUES(?,'segment','{}')",(${JSON.stringify(id)},))
c.execute("INSERT INTO class_events(session_id,payload) VALUES(?,'{}')",(${JSON.stringify(id)},))
c.execute("INSERT INTO class_cards VALUES('card',?,'{}')",(${JSON.stringify(id)},))
c.commit()
`);
 const output=fs.openSync('G:/MeetingAssistant/logs/history-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9226'},stdio:['ignore',output,output],windowsHide:true});
 let browser;
 try {
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9226');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop test endpoint did not start');
  const context=browser.contexts()[0];const page=context.pages()[0]||await context.waitForEvent('page');
  let manage=page.getByRole('button',{name:'管理课程：原生管理验证',exact:true});await manage.waitFor({timeout:30000});
  const data=await page.evaluate(async()=>{const r=await window.__TAURI_INTERNALS__.invoke('class_get_settings');return r.data_dir;});assert.equal(path.resolve(data),path.resolve(root,'data'));
  await manage.click();await page.getByRole('menuitem',{name:'重命名',exact:true}).click();await page.getByRole('textbox',{name:'新的课程名称',exact:true}).fill('原生课程已重命名');await page.getByRole('button',{name:'保存名称',exact:true}).click();await page.getByRole('dialog',{name:'重命名课程',exact:true}).waitFor({state:'hidden'});
  assert.equal(sql("import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); print(c.execute('SELECT title FROM class_sessions').fetchone()[0])"),'原生课程已重命名');assert(fs.existsSync(path.join(folder,'audio.mp4')));
  manage=page.getByRole('button',{name:'管理课程：原生课程已重命名',exact:true});
  await manage.click();await page.getByRole('menuitem',{name:'仅归档',exact:true}).click();await page.getByRole('button',{name:'仅归档',exact:true}).click();await page.getByRole('dialog',{name:'管理课程',exact:true}).waitFor({state:'hidden'});
  assert(fs.existsSync(path.join(folder,'audio.mp4')));
  assert.equal(sql("import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); print(c.execute('SELECT archived FROM class_sessions').fetchone()[0])"),'1');
  await page.getByRole('button',{name:'已归档',exact:true}).click();await manage.click();await page.getByRole('menuitem',{name:'恢复课程',exact:true}).click();await page.getByRole('button',{name:'恢复课程',exact:true}).click();
  await page.getByRole('dialog',{name:'管理课程',exact:true}).waitFor({state:'hidden'});await page.getByRole('button',{name:'返回记录',exact:true}).click();
  await manage.click();await page.getByRole('menuitem',{name:'彻底删除',exact:true}).click();await page.getByRole('radio').nth(1).check();await page.getByRole('checkbox',{name:'我确认永久删除这节课程及其全部文件'}).check();await page.getByRole('button',{name:'永久删除',exact:true}).click();
  await page.getByRole('dialog',{name:'管理课程',exact:true}).waitFor({state:'hidden'});assert(!fs.existsSync(folder));
  const counts=JSON.parse(sql("import sqlite3,sys,json; c=sqlite3.connect(sys.argv[1]); print(json.dumps({t:c.execute('SELECT COUNT(*) FROM '+t).fetchone()[0] for t in ['class_sessions','class_segments','class_events','class_cards','class_file_cleanup']}))"));assert(Object.values(counts).every(n=>n===0));
  fs.writeFileSync('G:/MeetingAssistant/logs/native-history-test.json',JSON.stringify({isolatedDataDir:data,checks:['old schema migration','course menu','rename persists and keeps original file','archive keeps original files','restore','permanent deletion removes files and all database rows'],remainingRows:counts,cloudCallsMade:false,microphoneOpened:false},null,2));
 } finally {if(browser)await browser.close();app.kill();fs.closeSync(output);}
})().catch(e=>{console.error(e);process.exitCode=1;});
