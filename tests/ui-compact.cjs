const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 const page=await browser.newPage({viewport:{width:560,height:720}}),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.goto('http://localhost:3118/?compact=1',{waitUntil:'networkidle'});
 await page.getByRole('heading',{name:'刚才，老师讲了什么？'}).waitFor();
 if(await page.getByRole('button',{name:'开始录课',exact:true}).count())throw Error('Compact popup unexpectedly contains recording controls');
 await page.getByRole('button',{name:/查看课堂演示/}).click();
 await page.locator('.cc-mode-tabs').getByRole('button',{name:'分层补听',exact:true}).click();

 await page.locator('.cc-run').click();await page.locator('.cc-layer summary').click();
 await page.getByText('AI辅助理解 · 不代表老师原话',{exact:true}).waitFor();
 if(await page.evaluate(()=>document.documentElement.scrollWidth>window.innerWidth))throw Error('Popup overflows horizontally');
 await page.screenshot({path:'G:/MeetingAssistant/build/课堂补听-快捷弹窗预览.png',fullPage:true});
 fs.writeFileSync('G:/MeetingAssistant/logs/ui-compact-test.json',JSON.stringify({checks:['compact popup','no duplicate recording controls','layered catch-up mode','no horizontal overflow'],pageErrors:errors},null,2));
 await browser.close();if(errors.length)process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1;});
