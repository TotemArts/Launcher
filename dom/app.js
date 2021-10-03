import { News } from "news";
import { GameDashboard } from "game-dashboard";
import { Settings } from "settings";

class App extends Element {
    news_items = [];
    servers = [];

    constructor() {
        super();
        Window.this.xcall("check_launcher_update", check_launcher_result);
        Window.this.xcall("get_servers", News.getServersCallback);
        load_news_feed();
    }


    pages = {
        news: <News items={this.news_items} />,
        game: <GameDashboard servers={this.servers} />
    };

    overlays = {
        settings: <Settings />
    };

    current = "game";

    componentDidMount() {
        document.$("#content").patch(<div id="content">{this.pages[this.current]}</div>);
    }

    render(props) {
        return <body model="Data">
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
        this.current = input;
    }
    ["on click at [overlay]"](evt, input) {
        document.$("#overlay").patch(<div id="overlay">{this.overlays[input.getAttribute("overlay")]}</div>);
    }
}

document.body.patch(<App />);