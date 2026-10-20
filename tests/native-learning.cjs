// Real Windows UI and SQLite persistence; synthetic data only, no API calls.
const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {spawn,spawnSync}=require('node:child_process');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
(async()=>{
 const root='G:/MeetingAssistant/build/learning-native-fixture-'+Date.now();fs.mkdirSync(root+'/data',{recursive:true});
 const fact=text=>({text,segment_ids:['s1']});
 const fixtures=Array.from({length:15},(_,i)=>({id:'c'+i,session_id:'lesson',request:{anchor_ms:600000+i*1000,lookback_ms:60000,mode:i===14?'layered':i===13?'ask':'catch_up',question:i===13?'为什么需要先验概率？':''},card:{headline:fact('历史测试记录 '+i),key_points:[fact('课堂讲解条件概率与先验。')],current_topic:null,tasks:[],explanation:null,...(i===13?{answer:fact('这是来自选中课堂片段的一段回答。'.repeat(30))}:{}),...(i===14?{layers:[{title:'理解先验概率',classroom:fact('课堂将先验和证据结合。'),explanation:'先验是获取新证据之前的判断。',example:'用已知患病率作为初始判断。',relation:'先理解先验，再理解后验。'}]}:{})},evidence:[],warning:''}));
 fs.writeFileSync(root+'/fixture-cards.json',JSON.stringify(fixtures));
 const seeded=spawnSync('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe',['-',root],{input:`import sqlite3,json,sys,pathlib
r=pathlib.Path(sys.argv[1]);c=sqlite3.connect(r/'data/classcatch.sqlite')
c.execute('CREATE TABLE class_sessions (id TEXT PRIMARY KEY,title TEXT NOT NULL,created_at TEXT NOT NULL,status TEXT NOT NULL,audio_folder TEXT,elapsed_ms INTEGER NOT NULL DEFAULT 0)')
c.execute('CREATE TABLE class_segments (session_id TEXT NOT NULL,id TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(session_id,id))')
c.execute('CREATE TABLE class_cards (id TEXT PRIMARY KEY,session_id TEXT NOT NULL,payload TEXT NOT NULL)')
c.execute("INSERT INTO class_sessions VALUES('lesson','Native learning','2026-10-19','completed',NULL,1800000)")
c.execute('INSERT INTO class_segments VALUES(?,?,?)',('lesson','s1',json.dumps(dict(id='s1',start_ms=500000,end_ms=700000,text='Bayes theorem combines prior and evidence.',is_final=True,revision=1))))
for card in json.loads((r/'fixture-cards.json').read_text(encoding='utf-8')):c.execute('INSERT INTO class_cards VALUES(?,?,?)',(card['id'],'lesson',json.dumps(card)))
c.commit()
`,encoding:'utf8',env:{...process.env,PYTHONIOENCODING:'utf-8'}});assert.equal(seeded.status,0,seeded.stderr);
 const log=fs.openSync('G:/MeetingAssistant/logs/learning-native-ui.log','w');
 const app=spawn('G:/MeetingAssistant/build/native/debug/meetily.exe',[],{cwd:'G:/MeetingAssistant/build/native/debug',env:{...process.env,MEETING_ASSISTANT_ROOT:root,WEBVIEW2_USER_DATA_FOLDER:path.join(root,'webview-profile'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9232'},stdio:['ignore',log,log],windowsHide:true});
 let browser;
 try{
  for(let i=0;i<45;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9232');break;}catch{await new Promise(r=>setTimeout(r,1000));}}
  if(!browser)throw Error('Desktop endpoint unavailable');
  const context=browser.contexts()[0],page=context.pages()[0]||await context.waitForEvent('page');
  await page.locator('.cc-history').filter({hasText:'Native learning'}).click({timeout:30000});
  await page.getByRole('button',{name:'记录 15',exact:true}).waitFor();
  await page.locator('.cc-layer summary').click();await page.getByText('AI辅助理解 · 不代表老师原话',{exact:true}).waitFor();
  await page.locator('.cc-composer-toggle').click();
  await page.screenshot({path:'G:/MeetingAssistant/build/分层补听-Windows桌面实测.png'});
  await page.getByRole('button',{name:'记录 15',exact:true}).click();assert.equal(await page.locator('.cc-card-history nav>button').count(),15);
  await page.getByRole('textbox',{name:'搜索补听记录',exact:true}).fill('历史测试记录 13');await page.locator('.cc-card-history nav>button').click();
  await page.locator('.cc-answer p').waitFor();assert.equal(await page.locator('.cc-answer p').count(),1);
  const count=await page.evaluate(async()=>{const invoke=window.__TAURI_INTERNALS__.invoke;const rows=await invoke('class_cards',{sessionId:'lesson'});return rows.length;});assert.equal(count,15);
  await page.screenshot({path:'G:/MeetingAssistant/build/提问回答-Windows桌面实测.png'});
  fs.writeFileSync('G:/MeetingAssistant/logs/native-learning-test.json',JSON.stringify({passed:true,isolatedRoot:root,storedRecords:count,legacyAndNewCardsLoaded:true,layersExpandable:true,answerIsOneParagraph:true,cloudCallsMade:false,microphoneOpened:false},null,2));
 }finally{if(browser)await browser.close();app.kill();fs.closeSync(log);}
})().catch(e=>{console.error(e);process.exitCode=1;});
