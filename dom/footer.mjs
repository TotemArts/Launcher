export class Footer extends Element {
    is_in_progress=true;
    current_action="Updating to version 3.9434";
    progress_percentage=0;
    update_available=false;
    
    
    render(props) {
        if (this.is_in_progress) {
            return <div {...props}>
                <div class="downloadBar">
                    <progressbar class="indicator" update_progress/>
                </div>
                <p class="nowrap" style="float:left;">
                    {this.current_action}: <span class="green">{this.progress_percentage}%</span>
                </p>
                <p overlay="verify" style="float:right;">more details</p>
            </div>;
        } else if (this.update_available) {
            return <div {...props}><div class="hexpand hflow vcenter"><p class="uppercase red hexpand vcenter">&#10005; Your game is not up-to-date!</p><button class="green" id="update">Update Game</button></div></div>;
        } else {
            return <div {...props}><div class="hexpand hflow vcenter"><p class="uppercase green hexpand vcenter">&#10003; Your game is up-to-date!</p><button class="green" id="launch">Launch to Menu</button></div></div>;
        }
    }
    ["on click at button#update"](evt, target) {
        view.start_download(onProgress, onUpdateDone, onUpdateErr);
        output_variables["current_action"] = "Updating game";
        show_overlay("verify.htm");
    }
}