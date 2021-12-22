export class Progress extends Object {
    current_action = "";
    has_progress_bar = false;
    hash_progress_done = "0";

    data;

    constructor(props) {
        super(props);
    }
    callback(progress) {
        globalThis.progress.data = progress;
        if(Object.keys(progress).length == 5) {
            var download_progress = (progress["download"][1] != 0) ? progress["download"][0] * 100 / progress["download"][1] : 0.0;

            if (progress["download"][1] != 0 && progress["hash"][1] == 0) {
              var processed_instructions = 100;
            } else {
              var processed_instructions = (progress["hash"][1] != 0) ? progress["hash"][0] * 100 / progress["hash"][1] : 0;
            }
      
            document.$("div#progress").componentUpdate({
              current_state: progress["action"],
              hash_progress: processed_instructions,
              hash_progress_done: progress["hash"][0],
              hash_progress_total: progress["hash"][1],
              download_progress: printf("%.1f", download_progress),
              download_speed: progress["download_speed"],
              patch_progress: (progress["patch"][1] != 0) ? progress["patch"][0] * 100 / progress["patch"][1] : 0,
              patch_progress_done: progress["patch"][0],
              patch_progress_total: progress["patch"][1]
            });
        }
    }

}