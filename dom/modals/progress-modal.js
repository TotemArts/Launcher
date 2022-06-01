export class ProgressModal extends Element 
{
    current_action = "";
    current_state = "";

    hash_progress = "0";
    hash_progress_done = "0";
    hash_progress_total = "0";
    download_files_done = "0";
    download_files_total = "0";
    download_progress = "0";
    download_speed = "0";
    patch_progress = "0";
    patch_progress_done = "0";
    patch_progress_total = "0";

    paused = false;
    in_progress = false;

    get_progressbar_style(width) {
      return printf("width:%s%%", width);
    }

    actionOrProgressbars() {
      if (this.hash_progress_total != "0") { 
        return <div>
          <p>Validating files: <span class="green hexpand">{Math.floor(this.hash_progress * 10) / 10}%</span>{this.hash_progress_done}/{this.hash_progress_total} files</p>
          <div class="downloadBar"><progressbar class="indicator" style={this.get_progressbar_style(this.hash_progress)}></progressbar></div>
          <p>Downloading: <span class="green hexpand">{Math.floor(this.download_progress * 10) / 10}%</span>{this.download_files_done}/{this.download_files_total} files, {this.download_speed}</p>
          <div class="downloadBar"><progressbar class="indicator" style={this.get_progressbar_style(this.download_progress)}></progressbar></div>
          <p>Applying: <span class="green hexpand">{Math.floor(this.patch_progress * 10) / 10}%</span>{this.patch_progress_done}/{this.patch_progress_total} files</p>
          <div class="downloadBar"><progressbar class="indicator" style={this.get_progressbar_style(this.patch_progress)}></progressbar></div>
        </div>;
      } else {
        return <p>{this.current_state}</p>
      }
    }

    getRightButtonText() {
      if (!this.in_progress) {
        return "Start";
      } else if (this.paused) {
        return "Resume";
      } else {
        return "Pause";
      }
    }

    render(props) {
        return <div id="progress" class="settings-modal">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">{this.current_action}</h3>
          <div class="minimize" close></div>
        </div>
        <div class="padding">
          {this.actionOrProgressbars()}
          <div>
            <button id="left" class="orange">Cancel</button>
            <button id="right" class="green">{this.getRightButtonText()}</button>
          </div>
        </div>
      </div>
    }

    componentDidMount() {
      if (globalThis.progress.data != undefined) {
        this.callback(globalThis.progress);
      }
      globalThis.callback_service.subscribe("progress", this, this.callback);
    }
  
    callback(progress_service) {
      var progress = progress_service.data;

      var download_progress = (progress.download.bytes.maximum != 0) ? progress.download.bytes.value * 100 / progress.download.bytes.maximum : 0.0;

      if (progress.download.bytes.maximum != 0 && progress.hash.maximum == 0) {
        var processed_instructions = 100;
      } else {
        var processed_instructions = (progress.hash.maximum != 0) ? progress.hash.value * 100 / progress.hash.maximum : 0;
      }

      this.componentUpdate({
        current_state: progress["action"],
        hash_progress: processed_instructions,
        hash_progress_done: progress.hash.value,
        hash_progress_total: progress.hash.maximum,
        download_files_done: progress.download.files.value,
        download_files_total: progress.download.files.maximum,
        download_progress: printf("%.1f", download_progress),
        download_speed: progress["download_speed"],
        patch_progress: (progress.patch.maximum != 0) ? progress.patch.value * 100 / progress.patch.maximum : 0,
        patch_progress_done: progress.patch.value,
        patch_progress_total: progress.patch.maximum
      });
    }
  
    componentWillUnmount() {
      globalThis.callback_service.unsubscribe("progress", this, this.callback);
    }

    ["on click at button#left"](evt, input) {
      console.log("cancelling download");
      Window.this.xcall("cancel_patcher");
    }

    ["on click at button#right"](evt, input) {
      if (!this.in_progress) {
        console.log("starting download");

        Window.this.xcall("start_download", globalThis.progress.callback, globalThis.progress.success_callback, globalThis.progress.failure_callback);
        this.in_progress = true;
        evt.target.content(<p>Pause</p>);
      } else if (this.paused) {
        console.log("resuming patcher");

        Window.this.xcall("resume_patcher");
        this.paused = false;
        evt.target.content(<p>Pause</p>);
      } else {
        console.log("pausing patcher");

        Window.this.xcall("pause_patcher");
        this.paused = true;
        evt.target.content(<p>Resume</p>);
      }
    }
}