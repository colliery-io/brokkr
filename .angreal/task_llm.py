import angreal
import os
import platform
import re




llm = angreal.command_group(name="llm", about="commands for gathering information for the llm")

brokkr_models_dir = os.path.join(
    angreal.get_root(),
    '..',
    "crates",
    "brokkr-models"
)


def find_files(base_dir, pattern):
    """Recursively find files matching the regex pattern."""
    matched_files = []
    regex = re.compile(pattern)
    for root, _, files in os.walk(base_dir):
        for file in files:
            if regex.match(file):
                matched_files.append(os.path.join(root, file))
    return matched_files

def concatenate_files(file_list, with_seperator=True):
    """Concatenate the contents of files with headers."""
    concatenated_content = ""
    for file_path in file_list:
        if with_seperator:
            concatenated_content += f"== {file_path} ==\n"
        with open(file_path, 'r') as f:
            concatenated_content += f.read() + "\n"
    return concatenated_content

def copy_to_clipboard(text):
    """Copy the given text to the clipboard using native tools."""
    if platform.system() == 'Darwin':  # macOS
        os.system(f'echo "{text}" | pbcopy')
    elif platform.system() == 'Linux':
        os.system(f'echo "{text}" | xclip -selection clipboard')
    else:
        raise NotImplementedError('This script only supports macOS and Linux.')




@llm()
@angreal.command(name="migrations", about="bring up backing services")
def get_migrations():
    
    # Find files matching 'up.sql' pattern
    files = find_files(os.path.join(brokkr_models_dir,'models'), '*')

    # Concatenate the files' contents
    concatenated_content = concatenate_files(files)

    # Copy the concatenated content to the clipboard
    copy_to_clipboard(concatenated_content)


@llm()
@angreal.command(name="models", about="bring up backing services")
def get_models():
    
    # Find files matching 'up.sql' pattern
    files = find_files(os.path.join(brokkr_models_dir,'src','models'), '.*\.rs')

    # Concatenate the files' contents
    concatenated_content = concatenate_files(files,with_seperator=False)

    # Copy the concatenated content to the clipboard
    copy_to_clipboard(concatenated_content)




