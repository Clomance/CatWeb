<?php
    // argc - количество аргументов
    // argv - аргументы
    if ($argc > 2){
        $method = trim($argv[1]);

        switch ($method){
            case "GET":
                $method = 0;
                break;
            case "POST":
                $method = 1;
                break;
            default:
                $method = -1;
                break;
        }

        for ($i = 2; $i < $argc; $i++){
            $arg = explode("=", $argv[$i]);
            $name = trim($arg[0]);
            $value = trim((string) $arg[1]);

            switch ($method){
                case 0:
                    $_GET[$name] = $value;
                    break;
                case 1:
                    $_GET[$name] = $value;
                    break;
                default:
                    break;
            }
        }
    }

    $working_directory = getenv("SOURCE_DIRECTORY");
    if ($working_directory){
        chdir($working_directory);
    }
?>