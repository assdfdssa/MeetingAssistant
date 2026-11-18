'use client';
import {useEffect,useRef,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {X,RotateCcw,Check,Loader2} from 'lucide-react';
import './prompt-settings.css';

type Item={id:string;title:string;description:string;text:string;default_text:string;max_length:number;allow_empty:boolean};
type Config={items:Item[];revision:string};
export default function PromptSettings({native,onClose,onSaved}:{native:boolean;onClose:()=>void;onSaved:(settings:{vocabulary:string;prompt_overrides:Record<string,string>})=>void}){
 const [config,setConfig]=useState<Config|null>(null),[draft,setDraft]=useState<Record<string,string>>({}),[selected,setSelected]=useState('catch_up');
 const [saving,setSaving]=useState(false),[error,setError]=useState(''),[notice,setNotice]=useState(''),[discard,setDiscard]=useState(false);
 const dialog=useRef<HTMLElement>(null),previousFocus=useRef<HTMLElement|null>(null);
 const dirty=!!config?.items.some(item=>draft[item.id]!==item.text);
 const item=config?.items.find(item=>item.id===selected);
 const invalid=config?.items.some(item=>(!item.allow_empty&&!draft[item.id]?.trim())||Array.from(draft[item.id]??'').length>item.max_length);
 const close=()=>{if(saving)return;if(dirty)setDiscard(true);else onClose();};
 useEffect(()=>{previousFocus.current=document.activeElement as HTMLElement;dialog.current?.querySelector<HTMLButtonElement>('button')?.focus();return()=>previousFocus.current?.focus();},[]);
 useEffect(()=>{let disposed=false;if(!native){setError('提示词设置需要在桌面程序中使用。');return;}
  invoke<Config>('class_get_prompts').then(value=>{if(disposed)return;setConfig(value);setDraft(Object.fromEntries(value.items.map(item=>[item.id,item.text])));}).catch(e=>{if(!disposed)setError(String(e));});return()=>{disposed=true;};
 },[native]);
 const save=async()=>{if(!config||invalid)return;setSaving(true);setError('');setNotice('');try{
  const settings=await invoke<{vocabulary:string;prompt_overrides:Record<string,string>}>('class_save_prompts',{texts:draft,revision:config.revision});
  onSaved(settings);
  const fresh=await invoke<Config>('class_get_prompts');setConfig(fresh);setDraft(Object.fromEntries(fresh.items.map(item=>[item.id,item.text])));setDiscard(false);
  setNotice('已保存。生成提示词从下一次请求生效；语音术语从下一次识别连接生效。');
 }catch(e){setError(String(e));}finally{setSaving(false);}};
 return <div className="cc-modal-backdrop"><section ref={dialog} role="dialog" aria-modal="true" aria-label="提示词设置" className="cc-modal cc-prompt-modal" onKeyDown={e=>{
  if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close();}
  if(e.key==='Tab'){const controls=Array.from(dialog.current?.querySelectorAll<HTMLElement>('button:not(:disabled),textarea:not(:disabled),select:not(:disabled),input:not(:disabled),summary')??[]);const first=controls[0],last=controls[controls.length-1];if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}}
 }}>
  <header><div><span className="cc-eyebrow">MAKE IT YOURS</span><h2>提示词设置</h2></div><button className="cc-icon-btn" aria-label="关闭提示词设置" disabled={saving} onClick={close}><X size={20}/></button></header>
  <p className="cc-muted">查看和编辑当前课堂功能实际使用的提示词。云端和本地生成共用三种课堂提示词；保存不会发送API请求。</p>
  {!config&&!error&&<p role="status">正在读取提示词…</p>}
  {config&&<><div className="cc-prompt-layout"><nav aria-label="提示词分类">{config.items.map(entry=><button key={entry.id} aria-pressed={selected===entry.id} disabled={saving} onClick={()=>{setSelected(entry.id);setDiscard(false);}}>{entry.title}<small>{draft[entry.id]!==entry.text?'未保存':entry.text!==entry.default_text?'已自定义':'默认'}</small></button>)}</nav>
   {item&&<div className="cc-prompt-editor"><label htmlFor="cc-prompt-text">{item.title}</label><p id="cc-prompt-description" className="cc-muted">{item.description}</p><textarea id="cc-prompt-text" aria-label="提示词原文" aria-describedby="cc-prompt-description" spellCheck={false} value={draft[item.id]??''} disabled={saving} onChange={e=>{setDraft({...draft,[item.id]:e.target.value});setNotice('');setDiscard(false);}}/>
    <div className="cc-prompt-meta"><span>{Array.from(draft[item.id]??'').length} / {item.max_length} 字{dirty?' · 有未保存修改':''}</span><button className="cc-text-btn" disabled={saving||draft[item.id]===item.default_text} onClick={()=>{setDraft({...draft,[item.id]:item.default_text});setNotice('已恢复本项默认内容，点击保存后生效。');}}><RotateCcw size={13}/>本项恢复默认</button></div>
   </div>}
  </div><details className="cc-prompt-help"><summary>输出结构与动态课堂内容说明</summary><p className="cc-muted">生成结果仍由程序解析为JSON卡片，请保留默认提示词中的字段名与结构。提问和分层补听的segment_ids应填[]；补听重点继续要求真实片段引用。可修改语言、详略、讲解风格和关注点，不能通过修改提示词增加程序尚未支持的输出字段。</p><p className="cc-muted">课堂时间范围、提问输入和转写正文由程序动态放入user消息中的request与transcript；它们是每次的课堂数据，不是固定提示词。API地址、模型、密钥在服务设置中管理；上游Meetily旧界面模板未用于当前课堂流程。</p></details></>}
  {error&&<p className="cc-error" role="alert">{error}</p>}{notice&&<p className="cc-notice" role="status">{notice}</p>}
  {discard?<div className="cc-prompt-discard"><span>修改尚未保存，是否放弃？</span><button className="cc-secondary" onClick={()=>setDiscard(false)}>继续编辑</button><button className="cc-stop" onClick={onClose}>放弃修改并关闭</button></div>:<div className="cc-modal-actions"><button className="cc-secondary" disabled={saving} onClick={close}>关闭</button><button className="cc-primary" disabled={saving||!config||!dirty||invalid||!native} onClick={save}>{saving?<Loader2 size={15} className="cc-spin"/>:<Check size={15}/>}保存提示词</button></div>}
 </section></div>;
}
