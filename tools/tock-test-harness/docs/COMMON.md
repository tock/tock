# Common Documentations

## Two Python Files: Build and Init

The reason to separate these into 2 files is to have distinct runner script. The build script shall
 be static, and the process to build Tock OS should be similar regardless of the board, or the 
embedded device. However, the flashing process and the test running procedure may differ from a
board to anther. Thus, the init script is just a simple integration script that will execute 
any script, or command, specified inside the test.config.yml file. As a result, the developer will 
have freedom to customize their own installation and testing procedures.
