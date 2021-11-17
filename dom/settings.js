export class Settings extends Element 
{
    x64bitVersion = Window.this.xcall("get_setting", "64-bit-version");
    skipMovies = Window.this.xcall("get_setting", "skipMovies");

    constructor() {
        super();
    }

    render() {

        return <div class="settings-window">
            <div class="titlebar">
                <h3 class="title center uppercase" style="width: *;">Settings</h3>
                <div class="close" close></div>
            </div>
            <div class="child-margin center padding">
                <button class={ this.skipMovies + " bool uppercase"} setting="skipMovies">Skip cinematics</button>
                <button class={ this.x64bitVersion + " bool uppercase"} setting="64-bit-version">64 bit mode</button>
                <button class="setting uppercase" setting="validate">Repair Game</button>
                <button class="setting uppercase" setting="reset">Clean Game Install</button>
                <button class="setting uppercase" setting="launcher-logs">Launcher Log Folder</button>
                <button class="setting uppercase" setting="game-logs">Game Log Folder</button>
            </div>
            <div class="center uppercase padding" style="font-size: 5pt;">This Application uses Sciter Engine (<a href="https://sciter.com" target="@system">https://sciter.com/</a>), copyright Terra Informatica Software, Inc.</div>
            <div class="center uppercase padding" style="font-size: 8pt;">Launcher version <output launcher_version/></div>
        </div>
    }

    ["on click at button.bool[setting]"](evt, input) {
        if (evt.target.classList.contains("true")) {
            evt.target.classList.remove("true");
            evt.target.classList.add("false");
            Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "false");
        } else if (evt.target.classList.contains("false")) {
            evt.target.classList.remove("false");
            evt.target.classList.add("true");
            Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "true");
        }
    }

    ["on click at button.setting[setting]"](evt, input) {
        const setting = evt.target.getAttribute("setting");

        switch(setting) {
            case "validate":
                onUpdateCallback('validate');
                break;
            case "reset":
                document.$('body').reset_game();
                break;
            case "launcher-logs":
                Window.this.xcall("open_launcher_logs_folder");
                break;
            case "game-logs":
                break;
        }
    }
}