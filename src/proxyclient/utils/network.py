import logging
import base64
import hashlib
from typing import Optional, Dict, Any
import requests
from tqdm import tqdm
from io import BytesIO

# Make sure this import points correctly to your generated protobuf file
# If schema_pb2.py is in proxyclient/proto_py/, this should work.
try:
    from ..proto_py import schema_pb2
except ImportError:
    # Fallback if the structure is slightly different (e.g., running script directly)
    # This might happen during development but isn't ideal for packaging.
    try:
        import proto_py.schema_pb2 as schema_pb2
    except ImportError:
        print('Error: Cannot find the schema_pb2.py module.')
        print("Please ensure it's generated and placed correctly in proxyclient/proto_py/")
        raise

# Ensure config is imported correctly
try:
    from .. import config
except ImportError:
    # Fallback if the structure is slightly different
    try:
        import config
    except ImportError:
        print('Error: Cannot find the config.py module.')
        print("Please ensure it's present in the proxyclient directory.")
        raise


logger = logging.getLogger(__name__)


class UploadStreamWithProgress(BytesIO):
    """
    Wrapper for BytesIO to provide a progress bar during requests upload.
    Ensures the tqdm progress bar is closed properly.
    """

    def __init__(self, buffer, desc='Uploading'):
        super().__init__(buffer)
        # Ensure buffer has length before creating tqdm
        buffer_len = 0
        try:
            buffer_len = len(buffer)
        except TypeError:
            logger.warning('Could not get length of buffer for progress bar.')

        self._tqdm = tqdm(
            total=buffer_len,
            unit='B',
            unit_scale=True,
            desc=desc,
            ncols=80,  # Adjust console width if needed
            leave=False,  # Remove bar after completion/error
        )
        self._initial_len = buffer_len  # Store initial length

    def read(self, size=-1):
        chunk = super().read(size)
        if self._tqdm and chunk:  # Check if tqdm exists and chunk is not empty
            self._tqdm.update(len(chunk))
        # Automatically close tqdm if read returns nothing (EOF)
        # Note: requests might not call read until EOF explicitly.
        # The finally block in send_request is more reliable for closing.
        # elif self._tqdm:
        #      self.close_tqdm()
        return chunk

    def __len__(self):
        """Return the original length for requests header calculation."""
        return self._initial_len

    def close_tqdm(self):
        """Safely closes the tqdm instance if it exists."""
        if hasattr(self, '_tqdm') and self._tqdm:
            try:
                self._tqdm.close()
            except Exception as e:
                # Log error if closing tqdm fails, but don't crash
                logger.debug(f'Minor error closing tqdm progress bar: {e}')
            self._tqdm = None  # Prevent future attempts/errors

    def close(self):
        """Closes tqdm and the underlying stream if it's not already closed."""
        self.close_tqdm()
        if not self.closed:  # Check before closing the BytesIO stream
            super().close()


def calculate_sha256(data: bytes) -> str:
    """Calculates the SHA256 hash of the given data."""
    return hashlib.sha256(data).hexdigest()


def send_request(
    server_url: str,
    request_proto: schema_pb2.ClientRequest,
    payload: Optional[bytes] = None,
    payload_desc: str = 'payload',
) -> requests.Response:
    """
    Sends the request to the bootloader server.

    Args:
        server_url: The base URL of the server.
        request_proto: The protobuf ClientRequest message.
        payload: The optional binary payload data.
        payload_desc: Description for the progress bar if payload exists.

    Returns:
        The requests.Response object.

    Raises:
        requests.exceptions.RequestException: If the request fails.
        Exception: For other unexpected errors (e.g., protobuf serialization).
    """
    endpoint = f'{server_url.rstrip("/")}{config.API_ENDPOINT}'
    logger.info(f'Sending request to {endpoint}')

    # Prepare headers
    try:
        serialized_req = request_proto.SerializeToString()
        req_base64 = base64.b64encode(serialized_req)
        headers = {'X-Client-Request': req_base64}
        # Let requests handle Content-Type for data uploads
        # Let requests calculate Content-Length when using file-like objects
    except Exception as e:
        logger.error(f'Failed to serialize or encode request protobuf: {e}')
        raise  # Re-raise as a critical setup error

    data_stream = None
    resp = None

    try:
        # Prepare data stream with progress if payload exists
        if payload:
            logger.info(f'Preparing payload ({len(payload)} bytes)')
            data_stream = UploadStreamWithProgress(payload, desc=payload_desc)
            data_to_send = data_stream
        else:
            logger.debug('No payload to send.')
            headers['Content-Length'] = '0'  # Explicitly set for no-body requests
            data_to_send = None

        logger.debug(f'POST {endpoint}')
        # Avoid logging full base64 header for security/readability
        log_headers = {k: (v if k != 'X-Client-Request' else f'{v[:10]}...') for k, v in headers.items()}
        logger.debug(f'Headers: {log_headers}')
        if data_to_send:
            logger.debug(f'Payload size: {len(data_to_send)}')  # Log the length __len__ reports

        # --- Send the request ---
        # Use try/finally specifically around requests.post to ensure TQDM cleanup
        try:
            resp = requests.post(
                endpoint,
                headers=headers,
                data=data_to_send,  # Pass the stream wrapper or None
                timeout=config.REQUEST_TIMEOUT,
                # stream=False by default, which is usually fine unless memory is extremely tight
            )
        finally:
            # --- Crucial Cleanup ---
            # Always ensure the TQDM progress bar is closed after the request attempt.
            # requests *should* close the underlying stream (data_stream) when done reading.
            if data_stream is not None:
                data_stream.close_tqdm()
                # Optionally, you could try closing the stream itself,
                # but UploadStreamWithProgress.close() checks if already closed.
                # It's generally safer to rely on requests to close the stream it read from.
                # data_stream.close() # This is likely safe due to the check in UploadStreamWithProgress.close

        # --- Process Response ---
        logger.info(f'Received response: {resp.status_code} {resp.reason}')
        resp.raise_for_status()  # Raise HTTPError for bad responses (4xx or 5xx) AFTER cleanup
        return resp

    # --- Specific Error Handling ---
    except requests.exceptions.Timeout:
        logger.error(f'Request timed out after {config.REQUEST_TIMEOUT} seconds connecting/reading from {endpoint}.')
        # TQDM should have been closed by the inner finally block.
        raise
    except requests.exceptions.ConnectionError as e:
        logger.error(f'Connection error: Failed to connect to {endpoint}. Is the server running? Details: {e}')
        # TQDM should have been closed by the inner finally block.
        raise
    except requests.exceptions.HTTPError as e:
        # This is raised by resp.raise_for_status() for 4xx/5xx responses
        logger.error(f'HTTP error response from server: {e}')
        if e.response is not None:
            logger.error(f'Response body: {e.response.text[:500]}...')  # Log snippet of error response
        # TQDM should have been closed by the inner finally block.
        raise
    except requests.exceptions.RequestException as e:
        # Catch other potential requests errors (e.g., invalid URL, redirect issues)
        logger.error(f'Request failed: An error occurred during the request to {endpoint}. Details: {e}')
        if e.response is not None:
            logger.error(f'Response status: {e.response.status_code}')
            logger.error(f'Response body: {e.response.text[:500]}...')  # Log snippet
        # TQDM should have been closed by the inner finally block.
        raise
    except Exception as e:
        # Catch any other unexpected errors during the process
        logger.error(f'An unexpected error occurred in send_request: {e}', exc_info=True)
        # Ensure cleanup happened if data_stream was created
        if data_stream is not None and hasattr(data_stream, 'close_tqdm'):
            data_stream.close_tqdm()  # Attempt cleanup again just in case
        raise  # Re-raise the caught exception
