import argparse
import logging
from pathlib import Path

import msgpack
import requests

import proto_py.schema_pb2 as schema_pb2
from proxyclient import config
from proxyclient.commands.base_command import BaseCommand
from proxyclient.utils import network

logger = logging.getLogger(__name__)


class BootCommand(BaseCommand):
    COMMAND_NAME = 'boot'
    COMMAND_HELP = 'Boot a Linux kernel via the proxy.'

    def add_arguments(self, parser: argparse.ArgumentParser):
        parser.add_argument(
            '-t', '--target', required=True, choices=config.SUPPORTED_ARCHS, help='Target architecture.'
        )
        parser.add_argument('--kernel-path', type=Path, help='Path to the kernel image (overrides default).')
        parser.add_argument('--initramfs-path', type=Path, help='Path to the initramfs image (overrides default).')
        parser.add_argument(
            '--dist-dir',
            type=Path,
            default=config.DEFAULT_DIST_DIR,
            help=f'Base directory for payload files (default: {config.DEFAULT_DIST_DIR})',
        )

    def run(self, args: argparse.Namespace):
        logger.info(f'Starting Linux boot command for target: {args.target}')

        # Determine file paths
        dist_dir = args.dist_dir.resolve()
        arch_defaults = config.LINUX_DEFAULTS.get(args.target)
        if not arch_defaults:
            # This shouldn't happen due to choices in argparse, but belt and braces
            logger.error(f"Unsupported architecture '{args.target}' - no default paths defined.")
            return 1  # Exit code

        kernel_path = args.kernel_path or (dist_dir / arch_defaults['kernel'])
        initramfs_path = args.initramfs_path or (dist_dir / arch_defaults['initramfs'])

        try:
            # Read files
            kernel_data = self._read_file(kernel_path, f'{args.target} kernel')
            initramfs_data = self._read_file(initramfs_path, f'{args.target} initramfs')
        except FileNotFoundError:
            return 1  # Error already logged by _read_file
        except OSError:
            return 1  # Error already logged by _read_file

        # Pack payload using msgpack
        logger.info('Packing kernel and initramfs using msgpack...')
        try:
            payload = msgpack.packb(
                {
                    'kernel_addr_r': kernel_data,
                    'ramdisk_addr_r': initramfs_data,
                }
            )
            payload_size = len(payload)
            payload_sha256 = network.calculate_sha256(payload)
            logger.info(f'Packed payload size: {payload_size} bytes')
            logger.info(f'Payload SHA256: {payload_sha256}')
        except Exception as e:
            logger.error(f'Failed to pack payload with msgpack: {e}')
            return 1

        # Build protobuf request
        logger.info('Building boot request protobuf message...')
        try:
            boot_request = schema_pb2.BootClientRequest(
                boot_type=schema_pb2.BootClientRequest.BootType.BOOT_TYPE_LINUX,
                payload_size=payload_size,
                payload_sha256=payload_sha256,
                # Add architecture info if your schema supports it
                # architecture=args.target
            )
            inner_request = schema_pb2.ClientRequest.ClientRequestInner(boot_request=boot_request)
            client_request = schema_pb2.ClientRequest(
                inner=inner_request,
                signature=None,  # Add signature logic if needed later
            )
        except Exception as e:
            logger.error(f'Failed to build protobuf message: {e}')
            return 1

        # Send request
        try:
            response = network.send_request(
                args.server_url, client_request, payload, payload_desc=f'{args.target} linux payload'
            )
            logger.info('Boot request sent successfully.')
            logger.info(f'Server Response Status: {response.status_code}')
            # Process response if needed (e.g., check for specific success message)
            logger.debug(f'Server Response Body:\n{response.text}')
            print('\nBoot request successful.')  # User-facing confirmation
            # Add more detailed success message based on response.text if possible
            print(f'Server Response: {response.status_code} {response.reason}')

        except requests.exceptions.RequestException as e:
            logger.error(f'Boot request failed: {e}')
            print(f'\nError: Boot request failed. Check logs for details.')
            return 1
        except Exception as e:
            logger.error(f'An unexpected error occurred: {e}', exc_info=True)
            print(f'\nError: An unexpected error occurred. Check logs for details.')
            return 1

        return 0  # Success exit code
