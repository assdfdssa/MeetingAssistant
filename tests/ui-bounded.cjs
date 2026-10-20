const {chromium}=require('C:/Users/YOUR_USER/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs=require('node:fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'}),results=[];
 try {
  for(const [width,height,compact] of [[1280,720,false],[1600,960,false],[820,620,false],[560,720,true],[740,900,false]]){
   const page=await browser.newPage({viewport:{width,height}});
   await page.goto(`http://localhost:3118/${compact?'?compact=1':''}`,{waitUntil:'networkidle'});
   await page.getByRole('button',{name:/查看课堂演示/}).click();
   await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();
   await page.locator('.cc-card').waitFor();
   const before=await page.locator('.cc-catchup').boundingBox();
   await page.locator('.cc-card').evaluate(card=>{for(let i=0;i<30;i++)card.appendChild(card.querySelector('.cc-point').cloneNode(true));});
   if(!compact)await page.locator('.cc-transcript-body').evaluate(body=>{for(let i=0;i<80;i++)body.appendChild(body.querySelector('.cc-segment').cloneNode(true));});
   const after=await page.locator('.cc-catchup').boundingBox();assert.equal(Math.round(before.height),Math.round(after.height),'Content must not grow the panel');
   if(!compact){
    await page.locator('.cc-transcript-body').hover();await page.mouse.wheel(0,1200);await page.waitForTimeout(200);
    assert(await page.locator('.cc-transcript-body').evaluate(el=>el.scrollTop>0));
    assert.equal(await page.locator('.cc-results').evaluate(el=>el.scrollTop),0);
   }
   const transcriptTop=compact?0:await page.locator('.cc-transcript-body').evaluate(el=>el.scrollTop);
   await page.locator('.cc-results').hover();await page.mouse.wheel(0,20000);await page.waitForTimeout(200);
   const actual=await page.evaluate(()=>{
    const app=document.querySelector('.cc-app'),res=document.querySelector('.cc-results');
    const r=res.getBoundingClientRect(),last=res.querySelector('.cc-card').lastElementChild.getBoundingClientRect();
    return {rootScrollTop:app.scrollTop,rootOverflow:app.scrollHeight>app.clientHeight+1,horizontalOverflow:app.scrollWidth>app.clientWidth+1,resultScrolled:res.scrollTop>0,lastPointVisible:last.bottom<=r.bottom+1&&last.bottom>r.top};
   });
   assert.equal(actual.rootScrollTop,0);assert(!actual.rootOverflow);assert(!actual.horizontalOverflow);assert(actual.resultScrolled);assert(actual.lastPointVisible);
   if(!compact)assert.equal(await page.locator('.cc-transcript-body').evaluate(el=>el.scrollTop),transcriptTop);
   if(width>800&&!compact){const record=await page.locator('.cc-recordbar').boundingBox();assert(record.y>=0&&record.y+record.height<height);}
   await page.screenshot({path:`G:/MeetingAssistant/build/固定工作区-${width}x${height}.png`});
   await page.getByRole('button',{name:'刚才没跟上，帮我补听',exact:true}).click();
   await page.waitForTimeout(150);assert.equal(await page.locator('.cc-results').evaluate(el=>el.scrollTop),0);
   results.push({width,height,compact,...actual});await page.close();
  }
  fs.writeFileSync('G:/MeetingAssistant/logs/ui-scroll-test.json',JSON.stringify({layout:'bounded independent panels',results},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
