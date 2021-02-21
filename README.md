# resonance-parrot
# Under Development #



## Binary: resonance-parrot


## Library: wavfile
Read and write WAV(RIFF waveform Audio Format) file.
Convert WAV data to f64 vec.

Format:
8,16,24bit PCM
32bit IEEE Float

Channel:
Mono or Stereo


## Library: kb-getch-sys
Rust ffi for _kbhit() and _getch().

### How to Build lib_kb_getch.lib(C Static library).
1.Boot x64 Native Tools Command Prompt for VS2019

2.Change Directory

`cd ./c_lib`

3.Compile

`cl /c lib_kb_getch.c`

4.Link

`lib lib_kb_getch.obj`