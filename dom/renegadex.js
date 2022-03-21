export class RenegadeXState extends Object {
  running = false;

  launch_to_server(server) {
    Window.this.xcall("launch_game", server, this.on_game_exit, this.on_game_fault);
    running = true;
  }

  

  on_game_exit() {
    this.running = false;
  }

  on_game_fault() {
    this.running = false;
  }
}