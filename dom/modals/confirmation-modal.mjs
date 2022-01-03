export class ConfirmationModal extends Element 
{
    render(props) {
        this.confirm = props.confirm_callback;
        return <div class="modal">
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

    ["on click at button.green"](evt, input) {
        this.confirm();
    }
}