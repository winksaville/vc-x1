# vc-x1 config file

vc-x1 config file document [0]

The two repos
- work doc[[4]]
- agent docs[[5]]
```toml
[repos]
work = "."
agent = ".claude"
```

The agent-session table
- items [[1]]
- result-lines [[2]]
- define col-width [[3]]
```toml
[agent-session]
items = "headers,user,assistant,tool,summary"
result-lines = 10
col-width = 68
```

[0]: ./vc-config.md#vc-config-settable-configuration-keys
[1]: ./vc-config.md#agent-sessionitems
[2]: ./vc-config.md#agent-sessionresult-lines
[3]: ./vc-config.md#agent-sessioncol-width
[4]: ./vc-config.md#reposwork
[5]: ./vc-config.md#reposbot
