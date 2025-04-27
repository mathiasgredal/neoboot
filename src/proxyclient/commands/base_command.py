import argparse
import logging
from abc import ABC, abstractmethod
from pathlib import Path

logger = logging.getLogger(__name__)


class BaseCommand(ABC):
    """Abstract base class for all proxy client commands."""

    # The command name (e.g., 'boot', 'chain')
    COMMAND_NAME: str = 'base'
    # Help text for the command
    COMMAND_HELP: str = 'Base command (should not be used directly)'

    def __init__(self, subparsers):
        self.parser = subparsers.add_parser(
            self.COMMAND_NAME,
            help=self.COMMAND_HELP,
            description=self.COMMAND_HELP,  # Use help as description for simplicity
        )
        self.add_arguments(self.parser)
        self.parser.set_defaults(func=self.run)  # Connect command name to run method

    @abstractmethod
    def add_arguments(self, parser: argparse.ArgumentParser):
        """Add command-specific arguments to the parser."""
        pass

    @abstractmethod
    def run(self, args: argparse.Namespace):
        """Execute the command's logic."""
        pass

    def _read_file(self, file_path: Path, description: str) -> bytes:
        """Helper to read a file with error handling and logging."""
        logger.debug(f'Reading {description} from: {file_path}')
        if not file_path.is_file():
            logger.error(f'{description.capitalize()} file not found at {file_path}')
            raise FileNotFoundError(f'{description.capitalize()} file not found: {file_path}')
        try:
            with open(file_path, 'rb') as f:
                content = bytearray()
                while True:
                    chunk = f.read(65536)  # Read in chunks
                    if not chunk:
                        break
                    content.extend(chunk)
                return bytes(content)  # Convert mutable bytearray back to immutable bytes
        except OSError as e:
            logger.error(f'Error reading {description} file {file_path}: {e}')
            raise
