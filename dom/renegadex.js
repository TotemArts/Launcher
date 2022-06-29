export class RenegadeXState extends Object {
  launch_to_server(server) {
    Window.this.xcall(
      "launch_game",
      server,
      this.on_game_exit,
      this.on_game_fault
    );
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", true);
    }
  }

  on_game_exit() {
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", false);
    }
  }

  on_game_fault() {
    if (globalThis && globalThis.callback_service) {
      globalThis.callback_service.publish("game_running", false);
    }
  }
}
