"""One-off refactor: make the production audio math independently testable."""
from pathlib import Path
folder=Path(r'G:\MeetingAssistant\meetily\frontend\src-tauri\src\classcatch')
path=folder/'cloud.rs'
source=path.read_text(encoding='utf-8')
begin=source.index('#[derive(Clone)]\nstruct Span')
end=source.index('pub async fn run<',begin)
tests=source.index('#[cfg(test)]')
dsp=source[begin:end]+source[tests:]
dsp=dsp.replace('struct Span {asr_start:u64,asr_end:u64,class_start:u64,class_end:u64}',
    'pub(super) struct Span {pub(super) asr_start:u64,pub(super) asr_end:u64,pub(super) class_start:u64,pub(super) class_end:u64}')
dsp=dsp.replace('fn map_time(', 'pub(super) fn map_time(')
dsp=dsp.replace('struct Downsampler {','pub(super) struct Downsampler {')
dsp=dsp.replace('    fn new()->Self','    pub(super) fn new()->Self').replace('    fn pcm(', '    pub(super) fn pcm(')
(folder/'dsp.rs').write_text(dsp,encoding='utf-8')
path.write_text('use super::dsp::{Downsampler,Span,map_time};\n'+source[:begin]+source[end:tests],encoding='utf-8')
