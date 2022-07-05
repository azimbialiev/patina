$publisher = "C:\Program Files\Mosquitto\mosquitto_pub.exe"
$subscriber = "C:\Program Files\Mosquitto\mosquitto_sub.exe"

$pubCommand = " -h localhost -p 1883 -t topic -m message -q 0 --repeat 100 --repeat-delay 0 --nodelay --id XXXX -V 5"
$subCommand = " -h localhost -p 1883 -t topic -V 5 --insecure -i XXXX -k 120 --nodelay -x 126 -q 0"

$subsProc = Start-Process -FilePath $subscriber -ArgumentList $subCommand -PassThru

for ($num = 1; $num -le 50; $num++){
    Start-Process -FilePath $publisher  -ArgumentList $pubCommand -NoNewWindow

}

Start-Sleep -s 30

$subsProc.Kill()