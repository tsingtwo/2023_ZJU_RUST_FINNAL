# 2023_ZJU_RUST_HW5

##食用方法
1.1 ```run --bin server <host> <port> [slaves'addr]```  e.g. ``` run --bin server 127.0.0.1 8180 127.0.0.1:8181 127.0.0.1:8182```
这里会告诉你logfile的位置

1.2 ```run --bin server <host> <port>``` host 和 port 是 1.1中 slave 的 host 和port e.g.``` run --bin server 127.0.0.1 8181``` ``` run --bin server 127.0.0.1 8182```
这里会告诉你从节点logfile的位置

1.3 ```cargo run --bin client address```  
主从节点都开 e.g. ```cargo run --bin client 127.0.0.1:8182```  ```cargo run --bin client 127.0.0.1:8181``` ```cargo run --bin client 127.0.0.1:8180```

##预期效果

只有在主节点的 set 和 delete 操作可以改变 redis 的内容， 主从节点都可以通过 set 访问 redis 的内容，主从节点的内容是同步的；
task1 的 logfile 也已经实现，task2 只考虑了 set get 和 del；
可以试一下多个主节点？不知道行不行（x

功能全部都实现了一遍，过滤词是 尊尼获嘉 和 Dell
（戴尔电脑真是垃圾，下午崩了我俩小时全在修电脑）
