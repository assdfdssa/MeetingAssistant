# Third-party components

This classroom fork is based on Meetily Community Edition v0.4.1. The upstream
MIT copyright notice is retained in Meetily-MIT.txt and the source repository.

Audio export uses the separate, unmodified FFmpeg executable identified as
`8.0.1-essentials_build-www.gyan.dev`. Its own `-L` output is retained in
FFmpeg-8.0.1-license-info.txt. This binary was built with `--enable-gpl` and
`--enable-version3`; GPL-3.0.txt is provided alongside it.

- FFmpeg project and source releases: https://ffmpeg.org/download.html
- Windows binary provider and build/source information: https://www.gyan.dev/ffmpeg/builds/
- FFmpeg licensing information: https://ffmpeg.org/legal.html

The bundled ONNX Runtime is version 1.22.0 from Microsoft. Its MIT notice is
retained in ONNXRuntime-MIT.txt; the original verified artifacts are preserved
in the project source tree.

Other source dependencies, their package versions, source references, and
license declarations remain recorded in Cargo.lock, Cargo.toml, the frontend
package files, and their corresponding dependency sources. This prototype's
notices are not a completed audit for commercial redistribution.
