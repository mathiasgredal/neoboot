import os
from pathlib import Path

# Default server URL
DEFAULT_SERVER_URL = os.environ.get('PROXYCLIENT_SERVER_URL', 'http://localhost:8080')
API_ENDPOINT = '/api/v1/rpc'

# Base directory for payload files (relative to project root or absolute)
# Assumes you run the client from proxyclient_project/
DEFAULT_DIST_DIR = Path(__file__).parent.parent.parent / 'dist'
# Or provide an absolute path if needed:
# DEFAULT_DIST_DIR = Path("/path/to/your/dist")

# Default payload paths (relative to DIST_DIR)
DEFAULT_WASM_PAYLOAD = 'wasm_oss/main.wasm'

# Linux defaults per architecture
LINUX_DEFAULTS = {
    'aarch64': {
        'kernel': 'linux/aarch64/Image',
        'initramfs': 'initramfs/aarch64/initramfs.cpio.gz',
    },
    'x86_64': {
        'kernel': 'linux/x86_64/bzImage',
        'initramfs': 'initramfs/x86_64/initramfs.cpio.gz',
    },
    'arm': {
        # Add paths for arm if different, e.g., zImage
        'kernel': 'linux/arm/zImage',
        'initramfs': 'initramfs/arm/initramfs.cpio.gz',
    },
}
SUPPORTED_ARCHS = list(LINUX_DEFAULTS.keys())

# Request timeout
REQUEST_TIMEOUT = 60  # seconds
