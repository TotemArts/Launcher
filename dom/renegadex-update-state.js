export class RenegadeXUpdateState extends Object {
    constructor(props) {
        super(props);
    }

    last_known_state = "";

    check_update() {
        Window.this.xcall("check_update", globalThis.renegadex.update.update_handler, globalThis.renegadex.update.error_callback);
    }

    update_handler(result) {
        var footer = document.$("div#footer");
        globalThis.renegadex.update.last_known_state = result;
        if (result === "resume") {
            Window.this.xcall("start_download", globalThis.progress.callback, globalThis.progress.success_callback, globalThis.progress.failure_callback);
        }

        if (footer !== null && footer !== undefined) {
            footer.componentUpdate({
                update_available: result
            });
        }
      }
  
      error_callback() {
        globalThis.renegadex.update.last_known_state = "error";

        if (footer !== null && footer !== undefined) {
            footer.componentUpdate({
                update_available: "error"
            });
        }
      }
}