export class FailureModal extends Element 
{
    render(props) {
        return <div class="modal">
        <div class="titlebar">
          <h3 class="title center uppercase" style="width: *;">{props.title}</h3>
          <div class="close" close></div>
        </div>
        <div class="child-margin vflow">
          <p>{props.message}</p>
          <button class="orange" close><p>{props.button}</p></button>
        </div>
      </div>
    }
}