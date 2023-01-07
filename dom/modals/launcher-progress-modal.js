export class LauncherProgressModal extends Element 
{
    download_progress = "0";
    download_speed = "0";

    get_progressbar_style(width) {
      return printf("width:%s%%%%", width);
    }

    render(props) {
      var download_progress = props.max * 100.0 / props.current;
        return <div id="progress" class="settings-modal">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">Updating launcher</h3>
          <div class="minimize" close></div>
        </div>
        <div class="padding">
          <p>Downloading: <span class="green hexpand">{Math.floor(download_progress * 10) / 10}%</span>{this.download_speed}</p>
          <div class="downloadBar"><progressbar class="indicator" style={this.get_progressbar_style(download_progress)}></progressbar></div>
        </div>
      </div>
    }
}