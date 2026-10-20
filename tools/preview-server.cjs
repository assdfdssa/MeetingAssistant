// Serve the already-built preview on this computer only. No dependencies,
// microphone access, credential access, or cloud calls are made by this server.
const http=require('node:http'),fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'../meetily/frontend/.next-production');
const mime={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.json':'application/json','.txt':'text/plain; charset=utf-8','.png':'image/png','.svg':'image/svg+xml','.ico':'image/x-icon','.woff2':'font/woff2'};
const port=Number(process.env.MEETING_ASSISTANT_PREVIEW_PORT||3118);
if(!fs.existsSync(path.join(root,'index.html'))){console.error('尚未生成界面，请先完成前端构建。');process.exit(1);}
http.createServer((req,res)=>{
  if(!['GET','HEAD'].includes(req.method)){res.writeHead(405);res.end();return;}
  let file;
  try{
    const name=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
    file=path.resolve(root,'.'+name.replace(/\\/g,'/'));
    if(file!==root&&!file.startsWith(root+path.sep)){res.writeHead(403);res.end();return;}
    if(file===root||fs.statSync(file).isDirectory())file=path.join(file,'index.html');
    const stat=fs.statSync(file);
    if(!stat.isFile())throw Error('Not a file');
    res.writeHead(200,{'Content-Type':mime[path.extname(file)]||'application/octet-stream','Content-Length':stat.size,'Cache-Control':'no-store','X-Content-Type-Options':'nosniff'});
    if(req.method==='HEAD'){res.end();return;}
    const stream=fs.createReadStream(file);stream.on('error',()=>res.destroy());stream.pipe(res);
  }catch{res.writeHead(404);res.end('Not found');}
}).on('error',e=>{console.error(e.code==='EADDRINUSE'?`端口 ${port} 已被使用，请关闭原预览或更换端口。`:e.message);process.exitCode=1;})
 .listen(port,'127.0.0.1',()=>console.log(`课堂补听界面预览：http://localhost:${port}（不录音、不调用API）`));
