export class Progress extends Element 
{
    current_action = "Baking beans";

    hash_progress = "0";
    hash_progress_done = "0";
    hash_progress_total = "0";
    download_progress = "0";
    download_speed = "0";
    patch_progress = "0";
    patch_progress_done = "0";
    patch_progress_total = "0";

    render(props) {

        return <div class="settings-window">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">{this.current_action}</h3>
          <div class="minimize" close></div>
        </div>
        <div class="padding">
          <p>Validating files: <span class="green hexpand">{this.hash_progress}%</span>{this.hash_progress_done}/{this.hash_progress_total} files</p>
          <div class="downloadBar"><progressbar class="indicator" value={this.hash_progress+"%"}></progressbar></div>
          <p>Dowloading: <span class="green hexpand">{this.download_progress}%</span>{this.download_speed}</p>
          <div class="downloadBar"><progressbar class="indicator" value={this.download_progress+"%"}></progressbar></div>
          <p>Applying: <span class="green hexpand">{this.patch_progress}%</span>{this.patch_progress_done}/{this.patch_progress_total} files</p>
          <div class="downloadBar"><progressbar class="indicator" value={this.patch_progress+"%"}></progressbar></div>
        </div>
      </div>
    }
}