class InputModal extends Element {
    this() {

    }

    render() {
        return <div class="username-window">
        <div class="titlebar">
          <h3 class="title.center uppercase" style="width: *;">Welcome back Commander!</h3>
          <div class="close" close></div>
        </div>
        <div class="child-margin vflow">
          <p class="uppercase">Username</p>
          <input type="text" class="username" maxlength="30"/>
          <button class="green">Submit</button>
          <button class="gray" close>Cancel</button>
        </div>
      </div>
    }

    ["on click at button.green"](evt, target) {
        set_username(this.$("input").value);
        close_overlay();
    }

    ["on keydown at input|text"](evt, target) {
        set_username(this.$("input").value);
        close_overlay();
    }
}