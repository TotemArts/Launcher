import { News } from "news.mjs";
import { GameDashboard } from "game-dashboard.mjs";
import { SettingsModal } from "settings-modal.mjs";
import { ConfirmationModal } from "confirmation-modal.mjs";
import { ProgressModal } from "progress-modal.mjs";
import { CallbackService } from "callback-service.mjs";
import { InputModal } from "input-modal.mjs";
import * as debug from "@debug";
import { LauncherProgressModal } from "./launcher-progress-modal.mjs";

globalThis.callback_service = new CallbackService();

debug.setUnhandledExeceptionHandler(function (err) {
  try {
    console.error("setUnhandledExceptionHandler:");
    console.error(printf("Caught exception: %s\n%V", err, err.stacktrace));;
  } catch (e) {
    console.error("setUnhandledExceptionHandler:");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));;
  }
});

function check_launcher_result(version) {
  globalThis.document.body.update_launcher(version);
}

class App extends Element {
  constructor() {
    super();
    Window.this.xcall("check_launcher_update", check_launcher_result);
    globalThis.username = Window.this.xcall("get_playername");
    globalThis.game_version = Window.this.xcall("get_game_version");
    globalThis.news_items = [];
    Window.this.xcall("fetch_resource", "https://ren-x.com/rss/1-recent-news.xml/", { "Referer": "https://ren-x.com/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache" }, globalThis.news_feed_callback, {});
  }

  pages = {
    news: <News />,
    game: <GameDashboard />
  };

  overlays = {
    settings: <SettingsModal />,
    username: <InputModal title="Welcome back commander!" key="Username" placeholder="" callback={this.set_username} />,
    progress: <ProgressModal />,
    clean_install: <ConfirmationModal title="Clean Game Install" message={<p>Are you sure you want to do this?<br />This will remove any additional content downloaded, and reset your settings!</p>} confirm="Clean!" confirm_callback={this.internal_clean_game} cancel="Uh..." />,
    validate_install: <ConfirmationModal title="Validate Game Install" message={<p>Are you sure you want to do this?<br />This will also reset your settings!</p>} confirm="Validate!" confirm_callback={this.internal_validate_game} cancel="Uh..." />,
  };

  current = "game";

  set_username(value) {
    try {
      Window.this.xcall("set_playername", value);
      globalThis.username = value;
      me = document.body;
      me.$("#content").patch(<div id="content">{me.pages[me.current]}</div>);
    } catch (e) {
      console.error(e);
    }
  }

  validate_game_modal() {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay">{this.overlays.validate_install}</div>);
      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch (e) {
      console.error(e);
    }
  }

  clean_game_modal() {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay">{this.overlays.clean_install}</div>);
      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch (e) {
      console.error(e);
    }
  }

  update_launcher(version) {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay"><ConfirmationModal title="A new version of the launcher is available!" message={<p>Do you want to download launcher version: {version}<br />You are currently on version: {Window.this.xcall("get_launcher_version")}</p>} confirm="Update!" confirm_callback={this.internal_update_launcher} cancel="I'd rather stay on this version" /></div>);
      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch(e) {
      console.error(e);
    }
  }

  internal_update_launcher() {
    globalThis.progress_callback = (array) => {
      var overlay = globalThis.document.$("#overlay");
      overlay.patch(<div id="overlay"><LauncherProgressModal current={array[0]} max={array[1]} /></div>);
      overlay.style["visibility"] = "visible";
      globalThis.document.$("div.menuEntries").state.disabled = true;
    };
    globalThis.failure_callback = (error) => {
      var overlay = globalThis.document.$("#overlay");
      overlay.patch(<div id="overlay"><ConfirmationModal title="The launcher update failed!" message={<p>Updating the launcher has failed:<br />{error}</p>} confirm="Uh...." confirm_callback={undefined} cancel="Uh..." /></div>);
      overlay.style["visibility"] = "visible";
      globalThis.document.$("div.menuEntries").state.disabled = true;
    };
    Window.this.xcall("update_launcher", globalThis.progress_callback, globalThis.failure_callback)
    console.log("Oh no, he wants to update!");
  }

  internal_validate_game() {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay">{document.body.overlays.progress}</div>);

      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch (e) {
      console.error(e);
    }
  }

  internal_clean_game() {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay">{document.body.overlays.progress}</div>);

      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch (e) {
      console.error(e);
    }
  }

  render(props) {
    return <body>
      <div id="header" style="background-color: #0D1721; margin-top: 2px;">
        <div class="headerSpacer left">
          <a class="facebook" href="https://www.facebook.com/RenXFirestorm/" target="@system"></a>
          <a class="twitter" href="https://twitter.com/renxgame" target="@system"></a>
          <a class="discord" href="https://ren-x.com/discord" target="@system"></a>
          <a class="youtube" href="https://www.youtube.com/channel/UCaiic-yEcwEv8VG6wRaHcDQ" target="@system"></a>
        </div>
        <div class="spacer" role="window-caption"></div>
        <div class="menuEntries">
          <div page="news">NEWS</div>
          <div class="current" page="game">PLAY GAME</div>
          <div overlay="settings">SETTINGS</div>
        </div>
        <div class="spacer" role="window-caption"></div>
        <div class="headerSpacer right">
          <button class="minimize" role="window-minimize"></button>
          <button class="maximize" role="window-maximize"></button>
          <button class="close" role="window-close"></button>
        </div>
      </div>

      <div id="overlay"></div>
      <div id="content">{this.pages[this.current]}</div>
    </body>
  }

  ["on click at [page]"](evt, target) {
    document.$("#content").patch(<div id="content">{this.pages[target.getAttribute("page")]}</div>);
    document.$("[page=" + this.current + "]").classList.remove("current");
    target.classList.add("current");
    this.current = target.getAttribute("page");
  }

  ["on click at [overlay]"](evt, target) {
    if (this.overlays[target.getAttribute("overlay")]) {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay">{this.overlays[target.getAttribute("overlay")]}</div>);

      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } else {
      console.error("no such overlay: " + target.getAttribute("overlay"));
    }
  }

  ["on click at #overlay [close]"](evt, target) {
    var overlay = document.$("#overlay");
    overlay.patch(<div id="overlay"></div>);
    overlay.style["visibility"] = "collapse";
    document.$("div.menuEntries").state.disabled = false;
  }

  ["on keydown"](evt, target) {
    if (evt.code == "KeyF5") {
      console.log(window.location);
      Window.this.load(window.location.href);
    }
  }
}

document.body.patch(<App />);