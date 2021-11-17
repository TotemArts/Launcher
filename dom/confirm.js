export class Confirm extends Element 
{
    render(props) {
        this.confirm = props.confirm_callback;
        return <div class="username-window">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">{props.title}</h3>
          <div class="close" close></div>
        </div>
        <div class="child-margin vflow">
          <p>{props.message}</p>
          <button class="green" close><p>{props.confirm}</p></button>
          <button class="gray" close><p>{props.cancel}</p></button>
        </div>
      </div>
    }

    ["on click at button.bool[setting]"](evt, input) {
        if (evt.target.classList.contains("true")) {
            evt.target.classList.remove("true");
            evt.target.classList.add("false");
            Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "false");
        } else if (evt.target.classList.contains("false")) {
            evt.target.classList.remove("false");
            evt.target.classList.add("true");
            Window.this.xcall("set_setting", evt.target.getAttribute("setting"), "true");
        }
    }

    ["on click at button.green"](evt, input) {
        this.confirm();
    }
}