import { News } from './news.js';
import { RenegadeXDashboard } from "./renegadex-dashboard.js";
import { SettingsModal } from "./modals/settings-modal.js";
import { ConfirmationModal } from "./modals/confirmation-modal.js";
import { ProgressModal } from "./modals/progress-modal.js";
import { Progress } from "./progress.js";
import { CallbackService } from "./callback-service.js";
import { InputModal } from "./modals/input-modal.js";
import * as debug from "@debug";
import { LauncherProgressModal } from "./modals/launcher-progress-modal.js";

globalThis.callback_service = new CallbackService();

debug.setUnhandledExeceptionHandler(function (err) {
  try {
    console.error("setUnhandledExceptionHandler:");
    console.error(printf("Caught exception: %s\n%V", err, err.stacktrace));
  } catch (e) {
    console.error("setUnhandledExceptionHandler:");
    console.error(printf("Caught exception: %s\n%V", e, e.stacktrace));
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
    globalThis.progress = new Progress();
    globalThis.news_items = [];
    Window.this.xcall("fetch_resource", "https://totemarts.games/forums/rss/1-recent-news.xml/", { "Referer": "https://totemarts.games/forums/forum/7-news/", "X-Requested-With": "XMLHttpRequest", "TE": "Trailers", "Pragma": "no-cache" }, globalThis.news_feed_callback, {});
  }

  pages = {
    news: <News />,
    game: <RenegadeXDashboard />
  };

  overlays = {
    settings: <SettingsModal />,
    username: <InputModal title="Welcome back commander!" key="Username" placeholder="" callback={this.set_username} />,
    ip: <InputModal title="Join by IP" key="Hey there sarge!" placeholder="IP:port" callback={this.join_server} />,
    progress: <ProgressModal />,
    clean_install: <ConfirmationModal title="Clean Game Install" message={<p>Are you sure you want to do this?<br />This will remove any additional content downloaded, and reset your settings!</p>} confirm="Clean!" confirm_callback={this.internal_clean_game} cancel="Uh..." />,
    validate_install: <ConfirmationModal title="Validate Game Install" message={<p>Are you sure you want to do this?<br />This will also reset your settings!</p>} confirm="Validate!" confirm_callback={this.internal_validate_game} cancel="Uh..." />,
  };

  current = "game";

  set_username(value) {
    try {
      Window.this.xcall("set_playername", value);
      globalThis.username = value;
      var me = document.body;
      me.$("#content").patch(<div id="content">{me.pages[me.current]}</div>);
    } catch (e) {
      console.error(e);
    }
  }

  join_server(server) {
    console.log("We're trying to join: " + server);
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

  update_game(version) {
    try {
      var overlay = document.$("#overlay");
      overlay.patch(<div id="overlay"><ConfirmationModal title="A new version of the game is available!" message={<p>Do you want to download game version: {version}<br />You are currently on version: {Window.this.xcall("get_remote_game_version")}</p>} confirm="Update!" confirm_callback={this.internal_update_game} cancel="I'd rather stay on this version" /></div>);
      overlay.style["visibility"] = "visible";
      document.$("div.menuEntries").state.disabled = true;
    } catch(e) {
      console.error(e);
    }
  }

  internal_update_game() {
    globalThis.failure_callback = (error) => {
      var overlay = globalThis.document.$("#overlay");
      overlay.patch(<div id="overlay"><ConfirmationModal title="The launcher update failed!" message={<p>Updating the launcher has failed:<br />{error}</p>} confirm="Uh...." confirm_callback={undefined} cancel="Uh..." /></div>);
      overlay.style["visibility"] = "visible";
      globalThis.document.$("div.menuEntries").state.disabled = true;
    };
    Window.this.xcall("start_download", globalThis.progress.callback, globalThis.progress.success_callback, globalThis.progress.failure_callback);
    console.log("Oh no, he wants to update!");
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
          <a class="website" href="https://totemarts.games" target="@system"></a>
          <a class="facebook" href="https://www.facebook.com/TotemArtsOfficial" target="@system"></a>
          <a class="twitter" href="https://twitter.com/TotemArtsStudio" target="@system"></a>
          <a class="discord" href="https://discord.gg/totemarts" target="@system"></a>
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
