# kritor_agent

这是一个由 [kritor](https://github.com/KarinJS/kritor) 到其它QQ机器人协议的代理服务，通信过程如下：
```
--------------------                     ----------------               ----------------------------------------------
| USER_APPLICATION |          ->         | kritor_agent |      ->       | other implementations(satori, onebot,.etc) |
--------------------                     ----------------               ----------------------------------------------
```

# 已实现的功能

### Satori

| Kritor | Satori | 备注 |
|--------|--------|------|
| 🟢 CoreService.GetVersion| kritor_agent:Satori |
| 🟢 GetCurrentAccount| login.get |
| 🟢 EventService.RegisterActiveListener| events | WebSocket 实现 |
| 🟢 MessageService.SendMessage | message.create |

