import importlib
import inspect
import logging
import pkgutil

from proxyclient.commands.base_command import BaseCommand

logger = logging.getLogger(__name__)

COMMANDS: dict[str, type[BaseCommand]] = {}


def discover_commands():
    """Dynamically discovers commands in the 'commands' package."""
    if COMMANDS:  # Only discover once
        return

    package = importlib.import_module(__name__)
    package_path = package.__path__
    prefix = package.__name__ + '.'

    logger.debug(f'Discovering commands in: {package_path}')

    for _, name, ispkg in pkgutil.iter_modules(package_path, prefix):
        if not ispkg:
            try:
                module = importlib.import_module(name)
                for item_name, item in inspect.getmembers(module):
                    if inspect.isclass(item) and issubclass(item, BaseCommand) and item is not BaseCommand:
                        # Instantiate the command class (passing subparsers happens later)
                        # We store the class itself for later instantiation.
                        cmd_instance_name = getattr(item, 'COMMAND_NAME', None)
                        if cmd_instance_name and cmd_instance_name != 'base':
                            logger.debug(f"Discovered command '{cmd_instance_name}' in {name}")
                            if cmd_instance_name in COMMANDS:
                                logger.warning(f"Duplicate command name '{cmd_instance_name}' found. Overwriting.")
                            COMMANDS[cmd_instance_name] = item  # Store the class
                        elif item is not BaseCommand:
                            logger.warning(f'Class {item_name} in {name} looks like a command but lacks COMMAND_NAME.')

            except ImportError as e:
                logger.error(f'Failed to import module {name}: {e}')
            except Exception as e:
                logger.error(
                    f'Error processing module {name}: {e}', exc_info=True
                )  # Log tracebacks for unexpected errors


discover_commands()  # Discover commands when this module is imported
