# win12-desktop
win12 桌面端，基于 Tauri 封装，将 Windows 12 网页版变成独立桌面应用。


## 关于调试

### 安装依赖
1.在`win12`项目根目录中安装依赖
```bash
npm install
```


### 启动调试

调试需要2个终端
在第一个终端中使用此命令启动Vite
```bash
npm run dev
```
在第二个终端中使用此命令启动Tarui
```bash
npm run tauri dev
```

如开发者已开启代理，可能会干扰调试，建议使用以下命令启动
```bash
 env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u all_proxy npm run tauri dev
 ```