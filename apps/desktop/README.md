# ProcessLens Desktop

The desktop shell is a Tauri 2 application. It invokes the same `processlens-core` APIs as the CLI and never shells out from JavaScript.

```powershell
npm install
npm run build
npx tauri dev
```

The Overview, Projects, Processes, Ports, and Network pages use only locally acquired data. Docker, History, and Settings remain disabled until their underlying local adapters exist.
