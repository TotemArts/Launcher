export class Footer extends Element {
    is_in_progress=true;
    current_action="Updating to version " + Window.this.xcall("get_remote_game_version");
    progress_percentage=0;
    update_available=false;
    
    get_progressbar_style(width) {
        return printf("width:%s%%", width);
    }

    render(props) {
        if (this.is_in_progress) {
            return <div {...props}>
                <div class="downloadBar">
                    <progressbar class="indicator" style={this.get_progressbar_style(this.progress_percentage)} />
                </div>
                <p class="nowrap" style="float:left;">
                    {this.current_action}: <span class="green">{this.progress_percentage}%</span>
                </p>
                <p overlay="progress" style="float:right;">more details</p>
            </div>;
        } else if (this.update_available) {
            return <div {...props}><div class="hexpand hflow vcenter"><p class="uppercase red hexpand vcenter">&#10005; Your game is not up-to-date!</p><button class="green" id="update">Update Game</button></div></div>;
        } else {
            return <div {...props}><div class="hexpand hflow vcenter"><p class="uppercase green hexpand vcenter">&#10003; Your game is up-to-date!</p><button class="green" id="launch">Launch to Menu</button></div></div>;
        }
    }

    componentDidMount() {
        if (globalThis.progress.data != undefined) {
          this.process_progress(globalThis.progress.data);
        }
        globalThis.callback_service.subscribe("progress", this, this.callback);
    }

    callback(progress) {
        this.componentUpdate({
            is_in_progress: true,
        });
        this.process_progress(progress);
    }

    process_progress(progress) {
        if(Object.keys(progress).length == 5) {
            var download_progress = (progress["download"][1] != 0) ? progress["download"][0] * 100 / progress["download"][1] : 0.0;

            if (progress["download"][1] != 0 && progress["hash"][1] == 0) {
              var processed_instructions = 100;
            } else {
              var processed_instructions = (progress["hash"][1] != 0) ? progress["hash"][0] * 100 / progress["hash"][1] : 0;
            }
            var patch_progress = (progress["patch"][1] != 0) ? progress["patch"][0] * 100 / progress["patch"][1] : 0;
            var current_state = progress["action"];
            var total_progress = (processed_instructions + download_progress + patch_progress) / 3;

            this.componentUpdate({
                current_action: current_state,
                progress_percentage: total_progress
            });
        }
    }

    componentWillUnmount() {
        globalThis.callback_service.unsubscribe("progress", this, this.callback);
      }

    ["on click at button#update"](evt, target) {
        view.start_download(onProgress, onUpdateDone, onUpdateErr);
        output_variables["current_action"] = "Updating game";
        show_overlay("verify.htm");
    }
}