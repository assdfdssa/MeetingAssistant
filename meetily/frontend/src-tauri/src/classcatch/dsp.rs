#[derive(Clone)]
pub(super) struct Span {pub(super) asr_start:u64,pub(super) asr_end:u64,pub(super) class_start:u64,pub(super) class_end:u64}
pub(super) fn map_time(spans:&[Span],t:u64)->u64 {
    map_boundary(spans,t,false)
}
pub(super) fn map_time_end(spans:&[Span],t:u64)->u64 {
    map_boundary(spans,t,true)
}
fn map_boundary(spans:&[Span],t:u64,end:bool)->u64 {
    if let Some(first)=spans.first(){if t<=first.asr_start{return first.class_start;}}
    for s in spans {if (if end{t>s.asr_start && t<=s.asr_end}else{t>=s.asr_start && t<s.asr_end}) {
        return s.class_start+(t-s.asr_start).min(s.class_end.saturating_sub(s.class_start));
    }}
    spans.last().map(|s|s.class_end).unwrap_or(0)
}

// Streaming 63-tap low-pass FIR before 48kHz -> 16kHz decimation; avoids aliasing
// and preserves phase/history across incoming audio windows.
pub(super) struct Downsampler {taps:Vec<f32>,history:std::collections::VecDeque<f32>,phase:usize}
impl Downsampler {
    pub(super) fn new()->Self {
        let mut taps:Vec<f32>=(0..63).map(|i|{let n=i as f32-31.;let fc=0.15;
            let sinc=if n==0.{2.*fc}else{(2.*std::f32::consts::PI*fc*n).sin()/(std::f32::consts::PI*n)};
            sinc*(0.54-0.46*(2.*std::f32::consts::PI*i as f32/62.).cos())}).collect();
        let sum:f32=taps.iter().sum();for x in &mut taps{*x/=sum;}
        Self{taps,history:std::collections::VecDeque::from(vec![0.;63]),phase:0}
    }
    pub(super) fn pcm(&mut self,audio:&[f32],rate:u32)->Result<Vec<u8>,String> {
        if rate!=48000 {return Err(format!("云端识别预期上游48kHz混合音频，实际{}Hz",rate));}
        let mut bytes=Vec::with_capacity(audio.len()*2/3);
        for &v in audio {
            self.history.pop_front();self.history.push_back(if v.is_finite(){v}else{0.});
            self.phase+=1;
            if self.phase==3 {self.phase=0;let s:f32=self.history.iter().zip(&self.taps).map(|(a,b)|a*b).sum();
                bytes.extend_from_slice(&((s.clamp(-1.,1.)*32767.) as i16).to_le_bytes());}
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn maps_pause_without_compressing_class_timeline(){
        let spans=vec![Span{asr_start:0,asr_end:1000,class_start:0,class_end:1000},Span{asr_start:1000,asr_end:2000,class_start:601000,class_end:602000}];
        assert_eq!(map_time(&spans,1500),601500);
        // At a pause boundary, starts belong to the resumed chunk while
        // exclusive ends belong to the chunk before the break.
        assert_eq!(map_time(&spans,1000),601000);
        assert_eq!(map_time_end(&spans,1000),1000);
        assert_eq!(map_time_end(&spans,0),0);
    }
    #[test] fn resampler_keeps_chunk_phase(){let mut ds=Downsampler::new();let a=ds.pcm(&vec![0.;481],48000).unwrap();let b=ds.pcm(&vec![0.;479],48000).unwrap();assert_eq!(a.len()+b.len(),640);}
    #[test] fn split_audio_matches_continuous_audio(){
        let audio:Vec<f32>=(0..4800).map(|i|(i as f32*0.131).sin()*0.5).collect();
        let full=Downsampler::new().pcm(&audio,48000).unwrap();
        let mut ds=Downsampler::new();let mut split=ds.pcm(&audio[..481],48000).unwrap();
        split.extend(ds.pcm(&audio[481..],48000).unwrap());assert_eq!(split,full);
    }
    #[test] fn filters_above_output_nyquist_before_decimation(){
        let energy=|hz:f32|{
            let audio:Vec<f32>=(0..4800).map(|i|(2.0*std::f32::consts::PI*hz*i as f32/48000.0).sin()*0.5).collect();
            let pcm=Downsampler::new().pcm(&audio,48000).unwrap();
            pcm.chunks_exact(2).skip(100).map(|b|{let x=i16::from_le_bytes([b[0],b[1]]) as f64;x*x}).sum::<f64>()
        };
        let speech=energy(1000.0);assert!(speech>1e8);assert!(energy(12000.0)<speech*0.01);
    }
}
