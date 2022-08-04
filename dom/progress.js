import { FailureModal } from "./modals/failure-modal.js";
import { SuccessModal } from "./modals/success-modal.js";

export class Progress extends Object {
  is_in_progress=false;
  current_action = "";

  total_progress_done = "0";
  progressbars = [];

  data;

  constructor(props) {
      super(props);
  }
  failure_callback(error) {
    globalThis.progress.is_in_progress = false;
    globalThis.callback_service.publish("progress", globalThis.progress);

    // todo: this should ideally go through app.js or something
    var message = <p>The update failed!<br/>{error}</p>;
    var overlay = document.$("#overlay");
    overlay.patch(<div id="overlay"><FailureModal title="Update failed" message={message} button="Negative!"></FailureModal></div>);
    overlay.style["visibility"] = "visible";
    document.$("div.menuEntries").state.disabled = true;
  }

  success_callback() {
    globalThis.progress.is_in_progress = false;
    globalThis.callback_service.publish("progress", globalThis.progress);

    globalThis.renegadex.update.check_update();

    // todo: this should ideally go through app.js or something
    var overlay = document.$("#overlay");
    overlay.patch(<div id="overlay"><SuccessModal title="Update succesfull" message="The update succeeded!" button="Affirmative!"></SuccessModal></div>);
    overlay.style["visibility"] = "visible";
    document.$("div.menuEntries").state.disabled = true;
  }

  callback(progress) {
      globalThis.progress.data = progress;
      globalThis.progress.process_progress(progress);
      globalThis.callback_service.publish("progress", globalThis.progress);
  }

  process_progress(progress) {
    if(Object.keys(progress).length == 5) {
      var download_progress = (progress.download.bytes.maximum != 0) ? progress.download.bytes.value * 100 / progress.download.bytes.maximum : 0.0;

      if (progress.download.bytes.maximum != 0 && progress.hash.maximum == 0) {
        var processed_instructions = 100;
      } else {
        var processed_instructions = (progress.hash.maximum != 0) ? progress.hash.value * 100 / progress.hash.maximum : 0;
      }
      var patch_progress = (progress.patch.maximum != 0) ? progress.patch.value * 100 / progress.patch.maximum : 0;
      
      this.is_in_progress = true;
      this.current_action = progress["action"];
      this.total_progress_done = (processed_instructions + download_progress + patch_progress) / 3;
    }
  }
}