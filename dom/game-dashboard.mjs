import { Servers } from "servers.mjs";

export class GameDashboard extends Element 
{
  username = "";
  game_version = "";

  this(props) {
    this.username = props.username;
    this.game_version = props.game_version;
  }

  render()
  {
    return <div class="grid">
  <div class="logo hflow">
    <div class="vflow vcenter">
      <p style="font-size: 7pt;">Welcome</p>
      <h1 class=".change" overlay="username.htm">
        {this.username}
      </h1>
    </div>
    <div style="width:*;"></div>
    <div class=".vflow.vcenter" style="text-align: right;">
      <p style="font-size: 7pt; line-height:100%;">Version</p>
      <h1>{this.game_version}</h1>
    </div>
  </div>
  <div class="left-margin"></div>
  <div class="right-margin"></div>
  <div class="footer-margin"></div>
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
</div>
  }
}