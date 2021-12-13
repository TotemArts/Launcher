export class LauncherProgressModal extends Element 
{
    download_progress = "0";
    download_speed = "0";

    get_progressbar_style(width) {
      return printf("width:%s%%", width);
    }

    render(props) {
      var download_progress = props.max * 100.0 / props.current;
        return <div id="progress" class="settings-modal">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">Updating launcher</h3>
          <div class="minimize" close></div>
        </div>
        <div class="padding">
          <p>Dowloading: <span class="green hexpand">{download_progress}%</span>{this.download_speed}</p>
          <div class="downloadBar"><progressbar class="indicator" style={this.get_progressbar_style(download_progress)}></progressbar></div>
        </div>
      </div>
    }
}