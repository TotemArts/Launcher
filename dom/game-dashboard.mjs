import { Servers } from "./servers.mjs";
import { devicePixels } from "@sciter";
import { Footer } from "./footer.mjs";

export class GameDashboard extends Element 
{
  render()
  {
    return  <div class="grid">
              <div class="logo hflow">
                <div class="vflow vcenter">
                  <p style="font-size: 7pt;">Welcome</p>
                  <h1 class="change" overlay="username">
                    {globalThis.username}
                  </h1>
                </div>
                <div style="width:*;"></div>
                <div class="vflow vcenter" style="text-align: right;">
                  <p style="font-size: 7pt; line-height:100%;">Version</p>
                  <h1>{globalThis.game_version}</h1>
                </div>
              </div>
              <div class="left-margin"></div>
              <div class="right-margin"></div>
              <Footer class="footer-margin"/>
              <div>
                <Servers/>
                <div class="chat-container">
                  <div class="titlebar">
                    <h3 class="title">Chat Lobby</h3>
                    <div class="spacer"></div>
                    <div class="nowrap" style="margin-right: 10dip;">connect disconnect users</div>
                  </div>
                  <div class="body mheight">
                    <div class="chat" readonly="true" spellcheck="false" style="behavior: htmlarea;">
                      <p>Chat is still under construction, our apologies!</p>
                      <br />
                      <div class="vflow"><hr /></div>
                    </div>
                    <div class="reply">
                      <input type="text" novalue="Type here..." enter="sendIrcMessage(this)" disabled/>
                    </div>
                  </div>
                </div>
              </div>
            </div>;
  }
  componentDidMount() {
    var target =  this.$(".grid > div:nth-child(5)");
    target.onsizechange = this.recalculateLayout;
  }
  recalculateLayout() {
    var target =  this;
    var min_width = target.children.length * devicePixels(500);
    var parent_width = target.state.box("width", "border", "parent");
    console.error("parent_width:");
    console.error(parent_width);
    if (parent_width >= min_width) {
      if (target.style["flow"] != "horizontal") {
        target.style["flow"] = "horizontal";
      }
    } else {
      if (target.style["flow"] != "vertical") {
        target.style["flow"] = "vertical";
      }
    }
  }
}