import argparse
import logging
import os
import sys

from proxyclient import config
from proxyclient.commands import COMMANDS  # Import the discovered commands dict

# Configure logging
log_level_str = os.environ.get('PROXYCLIENT_LOG_LEVEL', 'INFO').upper()
log_level = getattr(logging, log_level_str, logging.INFO)

# Basic configuration first, handler details later if needed
logging.basicConfig(
    level=log_level, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s', datefmt='%Y-%m-%d %H:%M:%S'
)
# Silence noisy libraries if desired
# logging.getLogger("requests").setLevel(logging.WARNING)
# logging.getLogger("urllib3").setLevel(logging.WARNING)

logger = logging.getLogger(__name__)


def setup_logging(verbosity: int):
    """Adjusts logging level based on verbosity flags."""
    base_loglevel = logging.INFO
    if verbosity == 1:
        base_loglevel = logging.DEBUG
    elif verbosity >= 2:
        # Could potentially add TRACE level or more debug here if needed
        base_loglevel = logging.DEBUG
    elif verbosity < 0:  # Quiet
        base_loglevel = logging.WARNING
    elif verbosity < -1:  # Quieter
        base_loglevel = logging.ERROR

    # Get root logger and set level
    root_logger = logging.getLogger()
    root_logger.setLevel(base_loglevel)

    # Update handler level if basicConfig was already called
    # This assumes the default StreamHandler added by basicConfig
    for handler in root_logger.handlers:
        handler.setLevel(base_loglevel)

    # Add more sophisticated handler configuration here if needed (e.g., file logging)

    logger.debug(f'Log level set to {logging.getLevelName(base_loglevel)} ({base_loglevel})')


def main():
    parser = argparse.ArgumentParser(
        description='Proxy client for the experimental bootloader.',
        formatter_class=argparse.RawTextHelpFormatter,  # Preserve formatting in help
    )

    parser.add_argument(
        '--server-url',
        default=config.DEFAULT_SERVER_URL,
        help=f'URL of the bootloader server (default: {config.DEFAULT_SERVER_URL})',
    )
    parser.add_argument(
        '-v', '--verbose', action='count', default=0, help='Increase verbosity level (-v for DEBUG, -vv for more).'
    )
    parser.add_argument(
        '-q', '--quiet', action='count', default=0, help='Decrease verbosity level (-q for WARNING, -qq for ERROR).'
    )
    # Version argument (optional)
    # parser.add_argument('--version', action='version', version='%(prog)s 0.1.0')

    # --- Subcommand Setup ---
    subparsers = parser.add_subparsers(
        dest='command',
        title='Available Commands',
        help='Sub-command help',
        required=True,  # Require a command to be specified
    )

    # Instantiate and register discovered commands
    if not COMMANDS:
        logger.error("No commands were discovered. Check the 'commands' directory and logs.")
        sys.exit(1)

    logger.debug(f'Registering commands: {list(COMMANDS.keys())}')
    for cmd_name, cmd_class in COMMANDS.items():
        try:
            # Instantiate the command, passing the subparsers collection
            cmd_instance = cmd_class(subparsers)
            logger.debug(f"Registered command '{cmd_name}' from class {cmd_class.__name__}")
        except Exception as e:
            logger.error(
                f"Failed to instantiate or register command '{cmd_name}' from {cmd_class.__name__}: {e}", exc_info=True
            )
            # Decide whether to exit or just skip the problematic command
            # sys.exit(1) # Or continue if some commands might still work

    # --- Argument Parsing and Execution ---
    args = parser.parse_args()

    # Setup logging level based on -v / -q flags AFTER parsing args
    verbosity = args.verbose - args.quiet
    setup_logging(verbosity)

    logger.debug(f'Parsed arguments: {args}')

    # Execute the function associated with the chosen subcommand
    # (set via `set_defaults(func=...)` in BaseCommand)
    if hasattr(args, 'func'):
        try:
            exit_code = args.func(args)
            sys.exit(exit_code or 0)  # Use returned exit code or default to 0
        except Exception as e:
            logger.error(f'An unhandled exception occurred during command execution: {e}', exc_info=True)
            print('\nError: An unexpected error occurred. Run with -v for details.', file=sys.stderr)  # noqa: T201
            sys.exit(1)
    else:
        # This should not happen if subparsers are 'required=True'
        logger.error('No command function found. This indicates an internal error.')
        parser.print_help()
        sys.exit(1)
