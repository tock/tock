import git
import logging
import os
import tockloader
import toml
from pathlib import Path

TOCK_BOARD_DIR = f'{Path.home()}/actions-runner/_work/tock/tock/boards/'
LIBTOCK_C_DIR = f'{Path.home()}/libtock-c/'
CI_TEST_DIR = f'{LIBTOCK_C_DIR}/examples/ci-tests/'
CONFIG_FILE = f'{Path.home()}/tock/tools/tock-test-harness/config.toml'
BOARD_CONFIG_FILE = 'ci_test.config.toml'

# This dictionary maps the board to the universal test
TEST_MOD_MAP = {
    'nrf52dk': 'Nrf52Test',
    'nrf52840dk': 'Nrf52840Test'
}

class Runner:
    """Runner class, container of build, flash, and test workflow
    
    board      - Board model, used to run tockloader
    comm_proc  - Communication protocol, used to run tockloader
    harness_id - Provides identity to the runner and will only execute commands
                 with corresponding harness ID in the test.config.toml
    """
    def __init__(self, **args):
        """Load TOML file in filename as the configuration"""
        self.home_dir = Path.home()
        self.path = 'path/to/board'
        self.board = 'board_model'
        self.test_mod = 'board_to_test'
        self.comm_proc = ''
        self.harness_id = '' # Considered free lancer if left blank
        self.log = self.setup_logger()
        self.args = args
        self.board_config = None # Initialized in load_config()

        with open(CONFIG_FILE, 'r') as config_toml:
            self.config = toml.load(config_toml)
            self.load_config()

        # If install not specified, run default install workflow
        if 'scripts' in self.board_config:
            if 'install' in self.board_config['scripts']:
                self.install_script = self.board_config['scripts']['install']
            
            # If test not specified, script should end
            if 'test' in self.board_config['scripts']:
                self.test_script = self.board_config['scripts']['test']
        
        if 'test' in self.board_config:
            self.test_config = self.board_config['test']


    def setup_logger(self):
        logging.basicConfig(
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        logger = logging.getLogger('Runner')
        logger.setLevel('INFO')
        
        return logger
    
    def load_config(self):
        """Read configuration and assign path to 'path' member variable"""
        self.path = TOCK_BOARD_DIR + self.config['env']['path'] + '/'
        self.board = self.config['env']['board']
        self.test_mod = self.config['env']['board_to_test']
        self.harness_id = self.config['env']['harness_id']
        self.comm_proc = self.config['env']['communication_protocol']
        
        with open(self.path + BOARD_CONFIG_FILE, 'r') as board_config_toml:
            self.board_config = toml.load(board_config_toml)

    def tock_build(self):
        """Build the Tock OS with the given configuration"""
        self.log.info('Initiating compilation.')
        os.chdir(self.path)
        return os.system('make') >> 8 # exit_code

    def tock_preinstall(self):
        """Check prerun sequence, run if exists."""
        if self.install_script and 'prerun' in self.install_script:
            # Execute prerun specification
            self.log.info('Initiating prerun specification.')
            return os.system(self.install_script['prerun']) >> 8 # exit_code
        else:
            self.log.info('No pre install script.')
            return 0 # exit_code

    def tock_postinstall(self):
        """Check prerun sequence, run if exists."""
        if self.install_script and 'postrun' in self.install_script:
            # Execute postrun specification
            self.log.info('Initiating postrun specification.')
            exit_code = os.system(self.install_script['postrun']) >> 8
            return exit_code
        else:
            self.log.info('No post install script.')
            return 0 # exit_code

    def tock_install(self):
        """Flash Tock OS bin to board with the given configuration
        
        Note: if configuration file does not specify 'install', this script will
              run the default installation, which is just 'make install'.
        """
        os.chdir(self.path)
        if exit_code := self.tock_preinstall() != 0:
            return exit_code

        self.log.info('Initiating installation.')
        
        if self.install_script and 'run' in self.install_script:
            if exit_code := os.system(self.install_script['run']) >> 8 != 0:
                return exit_code
        else:
            if exit_code := os.system('make install') >> 8 != 0:
                return exit_code

        if exit_code := self.tock_postinstall() != 0:
            return exit_code

        self.log.info('Installtion completed.')

        return 0 # exit code

    def app_build(self, apps):
        """Lookup the APPs listed in configuration in libtock-c and compile APPs
        """
        self.log.info('Compiling libtock-c APPs... \n')
        for app in apps:
            if os.path.exists(CI_TEST_DIR + app):
                os.chdir(CI_TEST_DIR + app)
                if exit_code := os.system('make') >> 8 != 0:
                    return exit_code

        return exit_code

    def app_install(self, apps):
        """Lookup the APPs listed in configuration in libtock-c and install to 
        the target board.

        This step depends on app_build. If build fail, then app_install should
        not be called
        """
        self.log.info('Installing libtock-c APPs... \n')
        for app in apps:
            if self.comm_proc != '':
                CMD = (f"tockloader install --board {self.board} " + 
                       f"--{self.comm_proc} " +
                       f'{CI_TEST_DIR}/{app}/build/{app}.tab')
                print('\n', CMD, '\n')
                if exit_code := os.system(CMD) >> 8 != 0:
                    return exit_code
            else:
                # exit_code
                exit_code = os.system(('tockloader install --board' + 
                            f'{self.board} ' + 
                            f'{CI_TEST_DIR}/{app}/build/{app}.tab')) >> 8
                if exit_code != 0:
                    return exit_code

        return exit_code

    def app_test(self, apps):
        """Lookup the APPs listed in configuration in libtock-c and install to 
        the target board.

        This step depends on app_build. If build fail, then app_install should
        not be called
        """
        self.log.info('Testing APPs... \n')
        for app in apps:
            # exit_code
            exit_code = os.system((f'python3 {CI_TEST_DIR}/{app}/test.py ' +
                                    f'{TEST_MOD_MAP[self.test_mod]}')) >> 8

            if exit_code != 0:
                return exit_code

        return exit_code

    def tock_pretest(self):
        """Check prerun sequence, run if exists."""
        if self.test_script and 'prerun' in self.test_script:
            # Execute prerun specification
            self.log.info('Initiating prerun specification.')
            return os.system(self.test_script['prerun']) >> 8 # exit_code
        else:
            self.log.info('No pre test script.')
            return 0 # exit_code

    def tock_posttest(self):
        """Check prerun sequence, run if exists."""
        if self.test_script and 'postrun' in self.test_script:
            # Execute postrun specification
            self.log.info('Initiating postrun specification.')
            return os.system(self.install_script['postrun']) >> 8 # exit_code
        else:
            self.log.info('No post test script.')
            return 0 # exit_code

    def tock_test(self):
        """Test workflow"""
        self.log.info('Initiating test workflow. \n')

        if exit_code := self.tock_pretest() != 0:
            return exit_code

        # Unpack test configuration and APPs installation
        if self.test_config != None:
            self.log.info('Updating libtock-c repository... \n')
            git.Repo(LIBTOCK_C_DIR).remotes.origin.pull() # git pull

            for harness_token in self.test_config:
                # Harness specifier for all harnesses
                if harness_token == 'all' or harness_token == self.harness_id:
                    apps = self.test_config[harness_token]['app']
                    if exit_code := self.app_build(apps) != 0:
                        return exit_code
                    if exit_code := self.app_install(apps) != 0:
                        return exit_code
                    if exit_code := self.app_test(apps) != 0:
                        return exit_code

        if exit_code := self.tock_posttest() != 0:
            return exit_code

        self.log.info('Test workflow complete.')
        return 0 # exit_code

    def run(self):
        """Top level run"""
        if self.args['build']:
            return self.tock_build()

        if self.args['install']:
            return self.tock_install()

        if self.args['test']:
            return self.tock_test()