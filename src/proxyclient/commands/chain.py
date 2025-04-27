import argparse
import logging
from pathlib import Path

import requests  # For exceptions

import proto_py.schema_pb2 as schema_pb2
from proxyclient import config
from proxyclient.commands.base_command import BaseCommand
from proxyclient.utils import network

logger = logging.getLogger(__name__)


class ChainCommand(BaseCommand):
    COMMAND_NAME = 'chain'
    COMMAND_HELP = 'Chainload a WASM payload via the proxy.'

    def add_arguments(self, parser: argparse.ArgumentParser):
        default_wasm = config.DEFAULT_DIST_DIR / config.DEFAULT_WASM_PAYLOAD
        parser.add_argument(
            '--payload-path',
            type=Path,
            default=default_wasm,
            help=f'Path to the WASM payload (default: {default_wasm})',
        )
        # Add --dist-dir if WASM default path depends on it
        parser.add_argument(
            '--dist-dir',
            type=Path,
            default=config.DEFAULT_DIST_DIR,
            help=f'Base directory for payload files (used for default payload path) (default: {config.DEFAULT_DIST_DIR})',
        )

    def run(self, args: argparse.Namespace):
        logger.info('Starting chainload command.')

        # Resolve payload path based on --dist-dir if default is used
        payload_path = args.payload_path
        if not args.payload_path.is_absolute() and args.payload_path == (
            config.DEFAULT_DIST_DIR / config.DEFAULT_WASM_PAYLOAD
        ):
            # Recalculate default path using the provided --dist-dir
            payload_path = args.dist_dir.resolve() / config.DEFAULT_WASM_PAYLOAD
        else:
            payload_path = payload_path.resolve()  # Resolve if absolute or custom relative path given

        try:
            # Read payload file
            payload_data = self._read_file(payload_path, 'WASM payload')
        except FileNotFoundError:
            return 1
        except IOError:
            return 1

        payload_size = len(payload_data)
        payload_sha256 = network.calculate_sha256(payload_data)
        logger.info(f'Payload size: {payload_size} bytes')
        logger.info(f'Payload SHA256: {payload_sha256}')

        # Build protobuf request
        logger.info('Building chainload request protobuf message...')
        try:
            # *** IMPORTANT: Check your schema_pb2 definition ***
            # The original snippet used 'boot_request=schema_pb2.ChainClientRequest(...)'
            # This looks like a potential typo. A ClientRequestInner typically has
            # one field set (oneof). Assuming it should be `chain_request` field:
            chain_request = schema_pb2.ChainClientRequest(
                payload_size=payload_size,
                payload_sha256=payload_sha256,
            )
            inner_request = schema_pb2.ClientRequest.ClientRequestInner(
                chain_request=chain_request  # Adjust if your schema uses a different field name
            )
            # If your schema *really* reuses the 'boot_request' field for chainloading,
            # then use the original line, but that's unusual protobuf design:
            # inner_request = schema_pb2.ClientRequest.ClientRequestInner(
            #     boot_request=chain_request # If schema reuses the field
            # )
            client_request = schema_pb2.ClientRequest(
                inner=inner_request,
                signature=None,  # Add signature logic if needed later
            )
        except AttributeError as e:
            logger.error(f"Protobuf schema error: Does ClientRequestInner have a 'chain_request' field? {e}")
            logger.error('Please check your `proto_py/schema_pb2.py` definition.')
            return 1
        except Exception as e:
            logger.error(f'Failed to build protobuf message: {e}')
            return 1

        # Send request
        try:
            response = network.send_request(args.server_url, client_request, payload_data, payload_desc='WASM payload')
            logger.info('Chainload request sent successfully.')
            logger.info(f'Server Response Status: {response.status_code}')
            logger.debug(f'Server Response Body:\n{response.text}')
            print('\nChainload request successful.')
            print(f'Server Response: {response.status_code} {response.reason}')

        except requests.exceptions.RequestException as e:
            logger.error(f'Chainload request failed: {e}')
            print(f'\nError: Chainload request failed. Check logs for details.')
            return 1
        except Exception as e:
            logger.error(f'An unexpected error occurred: {e}', exc_info=True)
            print(f'\nError: An unexpected error occurred. Check logs for details.')
            return 1

        return 0  # Success
