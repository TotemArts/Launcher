import { $ } from "@sciter";
import { News } from "news";
import { GameDashboard } from "game-dashboard";
import { Settings } from "settings";

export class Menu extends Element {
  pages = {
    news: <News/>,
    game: <GameDashboard servers={globalThis.server_list}/>
  };

  overlays = {
    settings: <Settings/>
  };

  current = "game";

  render(props) {

    return <div class="menuEntries">
      <div page="news">NEWS</div>
      <div class="current" page="game">PLAY GAME</div>
      <div overlay="settings">SETTINGS</div>
    </div>
  }

  ["on click at [page]"](evt,input) {
    document.$("#content").patch(<div id="content">{this.pages[input.getAttribute("page")]}</div>);
    document.$("[page="+this.current+"]").classList.remove("current");
    input.classList.add("current");
    this.current = input;
  }
  ["on click at [overlay]"](evt,input) { 
    document.$("#overlay").patch(<div id="overlay">{this.overlays[input.getAttribute("overlay")]}</div>);
  }
}