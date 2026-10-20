$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Speech
$folder='G:\MeetingAssistant\build\asr-samples'
New-Item -ItemType Directory -Path $folder -Force | Out-Null
$s=New-Object System.Speech.Synthesis.SpeechSynthesizer
$format=[System.Speech.AudioFormat.SpeechAudioFormatInfo]::new(16000,[System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen,[System.Speech.AudioFormat.AudioChannel]::Mono)
try {
    $s.SelectVoice('Microsoft Huihui Desktop')
    $s.SetOutputToWaveFile((Join-Path $folder 'chinese.wav'),$format)
    $s.Speak('今天我们学习条件概率。请大家在下周三之前提交实验报告。')
    $s.SetOutputToNull()
    $s.SelectVoice('Microsoft Zira Desktop')
    $s.SetOutputToWaveFile((Join-Path $folder 'english.wav'),$format)
    $s.Speak('Today we are learning conditional probability. Please submit your lab report before next Wednesday.')
    $s.SetOutputToNull()
    $s.SetOutputToWaveFile((Join-Path $folder 'mixed.wav'),$format)
    $s.SpeakSsml('<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="zh-CN"><voice name="Microsoft Huihui Desktop">今天我们讨论</voice><voice name="Microsoft Zira Desktop">Bayes theorem</voice><voice name="Microsoft Huihui Desktop">，也就是贝叶斯定理。这里的</voice><voice name="Microsoft Zira Desktop">posterior probability</voice><voice name="Microsoft Huihui Desktop">指的是后验概率，请记住这个概念。</voice></speak>')
} finally {$s.Dispose()}
Write-Output 'Created Chinese, English and sentence-internal code-switching samples locally. No cloud TTS.'
