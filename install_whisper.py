import urllib.request
import zipfile
import io
import os
import shutil

target_dir = r"d:\Загрузки\1\src-tauri\binaries"
os.makedirs(target_dir, exist_ok=True)

url = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.1/whisper-bin-x64.zip"
req = urllib.request.Request(
    url,
    headers={'User-Agent': 'Mozilla/5.0'}
)

print("Downloading whisper-bin-x64.zip...")
try:
    with urllib.request.urlopen(req) as response:
        zip_data = response.read()
        print("Downloaded", len(zip_data), "bytes")
        with zipfile.ZipFile(io.BytesIO(zip_data)) as z:
            for info in z.infolist():
                if info.filename.startswith("Release/"):
                    name = os.path.basename(info.filename)
                    if not name:
                        continue
                    
                    # We need whisper-cli.exe and all DLLs
                    if name == "whisper-cli.exe":
                        dest_name = "whisper-sidecar-x86_64-pc-windows-msvc.exe"
                        dest_path = os.path.join(target_dir, dest_name)
                        with z.open(info) as src, open(dest_path, "wb") as dest:
                            shutil.copyfileobj(src, dest)
                        print("Extracted sidecar exe:", dest_name)
                    elif name.endswith(".dll"):
                        dest_path = os.path.join(target_dir, name)
                        with z.open(info) as src, open(dest_path, "wb") as dest:
                            shutil.copyfileobj(src, dest)
                        print("Extracted DLL:", name)
except Exception as e:
    print("Error:", e)
