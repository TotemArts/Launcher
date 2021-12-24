export class Progress extends Object {
  is_in_progress=false;
  current_action = "";

  total_progress_done = "0";
  progressbars = [];

  data;

  constructor(props) {
      super(props);
  }
  failure_callback() {

  }

  success_callback() {

  }

  callback(progress) {
      globalThis.progress.data = progress;
      process_progress(progress);
      globalThis.callback_service.publish("progress", globalThis.progress);
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
}