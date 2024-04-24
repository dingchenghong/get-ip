对http://txt.go.sohu.com/ip/soip

发起请求，获取到自己的外网ip，把获取到的ip写入文件

下次再发起请求得到新的ip，与文件里的对比，发果发生变化了，刚通过邮件通知ip变更了

这里假设程序是在linux环境跑的，并且安装了heirloom-mailx命令