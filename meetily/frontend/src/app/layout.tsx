'use client';
import './globals.css';
import {usePathname} from 'next/navigation';
import Classroom from '@/components/classcatch/Classroom';
export default function RootLayout({children}:{children:React.ReactNode}){
 const path=usePathname();
 // Native popups load /index.html?compact=1, whose pathname is not '/'.
 // Every entry point of this cloud classroom build must bypass legacy setup.
 return <html lang="zh-CN"><body>{path==='/'?children:<Classroom/>}</body></html>;
}
