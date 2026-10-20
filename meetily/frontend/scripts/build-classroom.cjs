// Keep production export separate from the running development preview.
const {spawnSync}=require('node:child_process');
const result=spawnSync(process.execPath,[require.resolve('next/dist/bin/next'),'build'],{
  stdio:'inherit',env:{...process.env,MEETING_ASSISTANT_NEXT_DIR:'.next-production',NEXT_TELEMETRY_DISABLED:'1'}
});
if(result.error){console.error(result.error.message);process.exit(1);}
process.exit(result.status??1);
