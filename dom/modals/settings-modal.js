export class SettingsModal extends Element {
  x64bitVersion = Window.this.xcall("get_setting", "64-bit-version");
  skipMovies = Window.this.xcall("get_setting", "skipMovies");
  launcherVersion = Window.this.xcall("get_launcher_version");

  constructor() {
    super();
  }

  render() {

    return <div class="settings-modal">
      <div class="titlebar">
        <h3 class="title center uppercase" style="width: *;">Settings</h3>
        <div class="close" close></div>
      </div>
      <div class="child-margin center padding">
        <button class={this.skipMovies + " bool uppercase"} setting="skipMovies">Skip cinematics</button>
        <button class={this.x64bitVersion + " bool uppercase"} setting="64-bit-version">64 bit mode</button>
        <button class="setting uppercase" setting="validate">Repair Game</button>
        <button class="setting uppercase" setting="reset">Clean Game Install</button>
        <button class="setting uppercase" setting="launcher-logs">Launcher Log Folder</button>
        <button class="setting uppercase" setting="game-logs">Game Log Folder</button>
      </div>
      <div class="center uppercase padding" style="font-size: 7pt;">This Application uses <a href="https://sciter.com" target="@system">Sciter Engine</a>, copyright Terra Informatica Software, Inc.</div>
      <div class="center uppercase padding" style="font-size: 8pt;">Launcher version {this.launcherVersion}</div>
    </div>
  }

  ["on click at button.bool[setting]"](evt, input) {
    try {
      input.classList.toggle("true");
      input.classList.toggle("false");
      Window.this.xcall("set_setting", input.getAttribute("setting"), input.classList.contains("true")? "true": "false");
    } catch (e) {
      console.error(e);
    }
  }

  ["on click at button.setting[setting]"](evt, input) {
    try {
      const setting = evt.target.getAttribute("setting");

      switch (setting) {
        case "validate":
          document.body.validate_game_modal();
          break;
        case "reset":
          document.body.clean_game_modal();
          break;
        case "launcher-logs":
          Window.this.xcall("open_launcher_logs_folder");
          break;
        case "game-logs":
          Window.this.xcall("open_game_logs_folder");
          break;
      }
    } catch (e) {
      console.error(e);
    }
  }
}