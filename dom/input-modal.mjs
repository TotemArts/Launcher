export class InputModal extends Element {
  this(props) {
    let {title, key, placeholder, callback, ...rest} = props;
    this.title = title;
    this.key = key;
    this.placeholder = placeholder;
    this.callback = callback;
  }

  render() {
    return <div class="input-modal">
      <div class="titlebar">
        <h3 class="title.center uppercase" style="width: *;">{this.title}</h3>
        <div class="close" close></div>
      </div>
      <div class="child-margin vflow">
        <p class="uppercase">{this.key}</p>
        <input type="text" maxlength="30" placeholder={this.placeholder} />
        <button class="green">Submit</button>
        <button class="gray" close>Cancel</button>
      </div>
    </div>
  }

  ["on click at button.green"](evt, target) {
    this.submit();
  }

  ["on keydown at input|text"](evt, target) {
    if (evt.code == "KeyEnter") {
      this.submit();
    }
  }

  submit() {
    try{
      this.callback(this.$("input").value);
    } catch(e) {
      console.log("Error in callback:");
      console.error(e);
    }
    this.$("[close]").post(new Event("click", { bubbles: true }));
  }
}