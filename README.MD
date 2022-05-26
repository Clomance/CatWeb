Command arguments:
 - IPv4:port
 - SOURCE_DIRECTORY
 - PHP_PATH

Enviroment variables:
 - SOURCE_DIRECTORY - a web application source.
 - PHP_PATH - a path for php executable.
 - THREAD_LIMIT - not used
 - THREAD_STACK_MEMORY - defines the stack memory for each client thread

If any parameter is not set, its default value is used.
Command arguments have priority over enviroment variables.