export class RenegadeXState extends Object {
  running = false;

  launch_to_server(server) {
    Window.this.xcall(
      "launch_game",
      server,
      this.on_game_exit(this),
      this.on_game_fault(this)
    );
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", true);
    }
    this.running = true;
  }

  on_game_exit(state) {
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", false);
    }
    state.running = false;
  }

  on_game_fault(state) {
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", false);
    }
    state.running = false;
  }
}
