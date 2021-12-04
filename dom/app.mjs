import { News } from "news.mjs";
import { GameDashboard } from "game-dashboard.mjs";
import { Settings } from "settings.mjs";
import { Confirm } from "confirm.mjs";
import { Progress } from "progress.mjs";
import { CallbackService } from "callback_service.mjs";
import * as debug from "@debug";

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
    console.log("check_launcher_result callback");
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
        settings: <Settings />
    };

    current = "game";

    reset_game() {
        var overlay = document.$("#overlay");
        overlay.patch(<div id="overlay">{<Confirm title="Clean Game Install" message={<p>Are you sure you want to do this?<br/>This will remove any additional content downloaded!</p>} confirm="Clean!" confirm_callback={this.internal_reset_game} cancel="Uh..."/>}</div>);

        overlay.style["visibility"] = "visible";
        document.$("div.menuEntries").state.disabled = true;
    }

    internal_reset_game() {
        var overlay = document.$("#overlay");
        overlay.patch(<div id="overlay">{<Progress />}</div>);

        overlay.style["visibility"] = "visible";
        document.$("div.menuEntries").state.disabled = true;
    }

    componentDidMount() {
        document.$("#content").patch(<div id="content">{this.pages[this.current]}</div>);
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
            <div id="content"></div>
        </body>
    }

    ["on click at [page]"](evt, input) {
        document.$("#content").patch(<div id="content">{this.pages[input.getAttribute("page")]}</div>);
        document.$("[page=" + this.current + "]").classList.remove("current");
        input.classList.add("current");
        this.current = input.getAttribute("page");
    }

    ["on click at [overlay]"](evt, input) {
        var overlay = document.$("#overlay");
        overlay.patch(<div id="overlay">{this.overlays[input.getAttribute("overlay")]}</div>);

        overlay.style["visibility"] = "visible";
        document.$("div.menuEntries").state.disabled = true;
    }

    ["on click at #overlay [close]"](evt, input) {
        var overlay = document.$("#overlay");
        overlay.patch(<div id="overlay"></div>);
        overlay.style["visibility"] = "collapse";
        document.$("div.menuEntries").state.disabled = false;
    }
}

document.body.patch(<App />);