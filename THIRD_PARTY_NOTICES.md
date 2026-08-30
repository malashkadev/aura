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
