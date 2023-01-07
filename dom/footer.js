import { RenegadeXUpdateState } from "./renegadex-update-state.js";

export class Footer extends Element {
    progress;
    update_available="";

    this() {
      this.progress = Object.assign({}, globalThis.progress);
      if (globalThis.renegadex === undefined) {
        globalThis.renegadex = { update: new RenegadeXUpdateState() };
        globalThis.renegadex.update.check_update();
      } else if (globalThis.renegadex.update === undefined) {
        globalThis.renegadex.update = new RenegadeXUpdateState();
        globalThis.renegadex.update.check_update();
      } else {
        this.update_available = globalThis.renegadex.update.last_known_state;
      }
    }
    
    get_progressbar_style(width) {
        return printf("width:%s%%%%", width);
    }

    render(props) {
        if (this.progress.is_in_progress || this.update_available == "resume") {
            return <div id="footer" {...props}>
                <div class="downloadBar">
                    <progressbar class="indicator" style={this.get_progressbar_style(this.progress.total_progress_done)} />
                </div>
                <p class="nowrap" style="float:left;">
                    {this.progress.current_action}: <span class="green">{this.progress.total_progress_done}%</span>
                </p>
                <p overlay="progress" style="float:right;">more details</p>
            </div>;
        } else if (this.update_available == "update") {
          return <div id="footer" {...props}><div class="hexpand hflow vcenter"><p class="uppercase red hexpand vcenter"><span state-html="&#10005;"/> A new version is available: { Window.this.xcall("get_remote_game_version") }!</p><button class="green" id="update" overlay="progress">Update Game</button></div></div>;
        } else if (this.update_available == "full") {
          return <div id="footer" {...props}><div class="hexpand hflow vcenter"><p class="uppercase red hexpand vcenter"><span state-html="&#10005;"/> The game is not installed, version available: { Window.this.xcall("get_remote_game_version") }!</p><button class="green" id="update" overlay="progress">Install Game</button></div></div>;
        } else if (this.update_available == "") {
          return <div id="footer" {...props}>Attempting to reach the download servers for version information!</div>;
        } else if (this.update_available == "up_to_date") {
            return <div id="footer" {...props}><div class="hexpand hflow vcenter"><p class="uppercase green hexpand vcenter"><span state-html="&#10003;"/> Your game is up-to-date!</p><button class="green" id="launch">Launch to Menu</button></div></div>;
        } else if (this.update_available == "error") {
          return <div id="footer" class="red" {...props}>Error while trying to reach the download servers for version information!</div>;
        }
    }

    componentDidMount() {
        if (globalThis.progress != undefined) {
          this.callback(globalThis.progress);
        }
        globalThis.callback_service.subscribe("progress", this, this.callback);
    }

    callback(progress) {
        this.componentUpdate({
            progress: Object.assign({}, progress),
        });
    }

    componentWillUnmount() {
        globalThis.callback_service.unsubscribe("progress", this, this.callback);
      }

    ["on click at button#update"](evt, target) {
        Window.this.xcall("start_download", globalThis.progress.callback, globalThis.progress.success_callback, globalThis.progress.failure_callback);
    }
}