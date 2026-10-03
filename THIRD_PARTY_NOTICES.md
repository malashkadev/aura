# Third-Party Notices

Aura is licensed under AGPL-3.0-only. Optional CUDA acceleration downloads
additional proprietary NVIDIA runtime components after installation. Those
components are not part of Aura and are not licensed under the AGPL.

## NVIDIA CUDA Toolkit 11.8

When compatible system copies are unavailable, Aura downloads the CUDA runtime
libraries cuBLAS 11.11.3.6 and cuFFT 10.9.0.58 directly from
`developer.download.nvidia.com`. Use and distribution are governed by the NVIDIA
CUDA Toolkit license:

https://docs.nvidia.com/cuda/archive/11.8.0/eula/index.html

## NVIDIA cuDNN 8.5.0

When a compatible system copy is unavailable, Aura downloads the cuDNN 8.5.0.96
inference runtime directly from `developer.download.nvidia.com`. Use and
distribution are governed by the NVIDIA cuDNN Software License Agreement:

https://docs.nvidia.com/deeplearning/cudnn/archives/cudnn-850/sla/index.html

The CUDA setup flow presents both agreements before installing optional CUDA
support. Compatible libraries already available through the process `PATH` are
reused; downloaded runtime files are installed in Aura's private application
data directory. Both layouts are used only by Aura's local transcription
processes.

## llama.cpp (b11179)

The local text-editing feature bundles prebuilt `llama-server` binaries for
the CPU and Vulkan backends in `src-tauri/binaries/llm-cpu/` and
`src-tauri/binaries/llm-vulkan/`. llama.cpp is licensed under the MIT
License:

https://github.com/ggml-org/llama.cpp/blob/master/LICENSE

These builds include the LLVM OpenMP runtime (`libomp.dll`); the license text
ships next to the binaries as `LICENSE-LLVM-OpenMP` (Apache License 2.0 with
LLVM exceptions).

## Qwen3 Model Weights (optional download)

On user request, Aura downloads GGUF-quantized Qwen3 editing models from
Hugging Face (the official `Qwen/Qwen3-4B-GGUF` repository and the
`mradermacher/Qwen3-0.6B-GGUF` / `Qwen3-1.7B-GGUF` mirrors) into Aura's
private application data directory. The weights are not bundled with the
installer and are not licensed under the AGPL; Qwen3 model weights are
licensed under the Apache License 2.0:

https://www.apache.org/licenses/LICENSE-2.0
