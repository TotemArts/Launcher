class ServersTable extends Element {

  currentItem = null;
  selectedItems;
  props;
  list;

  this(props) {
    super.this?.(props);
    this.props = props;

    if (globalThis.server_list) {
      console.log("servers in globalThis.server_list");
      this.list = globalThis.server_list.get_servers();
    } else {
      this.list = [];
    }
  }

  itemAt(at) {
    return this.list[at];
  }
  totalItems() {
    return this.list.length;
  }
  indexOf(item) {
    return this.list.indexOf(item);
  }

  render() {

    console.log("render ServerTable");
    let list = [];
    let totalItems = this.totalItems();
    let { currentItem, selectedItems } = this;


    for (let index = 0; index <= totalItems; ++index) {
      let item = this.itemAt(index);
      if (item) list.push(this.renderItem(item, item === currentItem, selectedItems?.has(item)));
    }
    return this.renderList(list);
  }

  componentDidMount() {
    globalThis.callback_service.subscribe("servers", this, this.callback);
  }

  callback(data) {
    console.log("Servers callback");
    console.log(this);
    this.componentUpdate({ list: data });
  }

  componentWillUnmount() {
    globalThis.callback_service.unsubscribe("servers", this, this.callback);
  }

  renderList(items) {
    return <table class="servers" {...this.props}>
      <thead>
        <tr>
          <th id="locked" class="locked sortable"></th>
          <th id="name" class="sortable">Server Name</th>
          <th id="map" class="sortable">Map</th>
          <th id="players" class="sortable">Players</th>
          <th id="latency" class="sortable">Ping</th>
        </tr>
      </thead>
      <tbody>
        {items}
      </tbody>
    </table>;
  }

  renderItem(item, isCurrent, isSelected) {
    return <tr key={item.key}>
      <th class={this.isLocked(isCurrent)}></th>
      <th>{item["Name"]}</th>
      <th>{item["Current Map"]}</th>
      <th>{item["Players"]}</th>
      <th>Ping</th>
    </tr>;
  }

  isLocked(isCurrent) {
    if (isCurrent)
      return "locked";
    return "";
  }

  itemOfElement(element) {
    return this.itemAt(element.elementIndex);
  }

  onkeydown(evt) {
    console.log("onkeydown");
    switch (evt.code) {
      case "KeyDOWN":
        if (!this.currentItem) {
          this.componentUpdate({ currentItem: this.itemOfElement(0) });
        } else {
          let index = this.indexOf(this.currentItem);
          if (++index < this.totalItems()) {
            this.componentUpdate({ currentItem: this.itemAt(index) });
            //this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyUP":
        if (!this.currentItem) {
          this.componentUpdate({ currentItem: this.itemAt(this.list.length - 1) });
        } else {
          let index = this.indexOf(this.currentItem);
          if (--index >= 0) {
            this.componentUpdate({ currentItem: this.itemAt(index) });
            //this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyEND":
        this.currentItem = this.itemAt(this.totalItems() - 1);
        //this.vlist.navigate("end");
        break;
      case "KeyHOME":
        this.currentItem = this.itemAt(0);
        //this.vlist.navigate("start");
        break;
      default:
        return false;
    }
    this.post(new Event("input", { bubbles: true }));
    return true;
  }

  setCurrentOption(child) {
    console.log("setCurrentOption");
    console.log(child);
    let option;
    for (let node = child; node; node = node.parentElement) {
      if (node.parentElement === this) {
        option = node;
        break;
      }
    }
    if (option) {
      console.log(this.itemOfElement(option));
      this.componentUpdate({ currentItem: this.itemOfElement(option) });
      this.post(new Event("input", { bubbles: true }));
      return true;
    }
  }

  ["on mousedown at tr"](evt) {
    console.log("mousedown");
    if (evt.button == 1) {
      console.log("mousedown click");
      this.setCurrentOption(evt.target);
    }
  }

  get value() {
    if (!this.currentItem) return undefined;
    return this.currentItem;
  }
}

class ServerList {
  minimum_players = 0;
  maximum_players = 64;
  current_players = 0;
  sortBy = "Players";
  game_version = "5.48.145";

  servers = [];

  constructor() {
    Window.this.xcall("get_servers", this.servers_callback);
  }

  refresh_servers() {
    Window.this.xcall("get_servers", globalThis.server_list.servers_callback);
  }

  servers_callback(data) {
    globalThis.server_list.servers = data;
    globalThis.server_list.notify_subscribers();
  }

  set_minimum_players(players) {
    globalThis.server_list.minimum_players = players;
    notify_subscribers();
  }

  set_maximum_players(players) {
    globalThis.server_list.maximum_players = players;
    notify_subscribers();
  }

  notify_subscribers() {
    globalThis.callback_service.publish("servers", globalThis.server_list.get_servers());
  }

  get_servers() {
    const list = [];

    /* Example entry of this.servers
      {
        "Name": "blabla",
        "Current Map": "CNC-LakeSide",
        "Bots": 1,
        "Players": 0,
        "Game Version": "5.48.145",
        "Variables": {
          "bPassworded": false,
          "bAllowPrivateMessaging": true,
          "bRanked": true,
          "Game Type": 1,
          "Player Limit": 10,
          "Vehicle Limit": 11,
          "bAutoBalanceTeams": false,
          "Team Mode": 6,
          "bSpawnCrates": true,
          "CrateRespawnAfterPickup": 35.0,
          "Time Limit": 0
        },
        "Port": 7777,
        "IP": "00.00.00.143"
      },
    */
    globalThis.server_list.current_players = 0;
    for (const server of globalThis.server_list.servers) {
      globalThis.server_list.current_players += server["Players"];
      if (server["Players"] >= globalThis.server_list.minimum_players &&
        server["Players"] <= globalThis.server_list.maximum_players &&
        (!globalThis.server_list.same_version || server["Game Version"] == globalThis.server_list.game_version)) {
        list.push(server);
      }
    }
    console.log("total players: " + globalThis.server_list.current_players);
    return list;
  }
}

export class Servers extends Element {
  server_list = new ServerList();

  this() {
    if(!globalThis.server_list)
      globalThis.server_list = new ServerList();
    this.server_list = globalThis.server_list;
  }

  componentDidMount() {
    globalThis.callback_service.subscribe("servers", this, this.callback);
  }

  callback(server_list) {
    console.log("Servers callback called Sjdkfja;");
    var server_list_clone = Object.assign({}, globalThis.server_list);
    this.componentUpdate({ server_list: server_list_clone });
  }

  componentWillUnmount() {
    globalThis.callback_service.unsubscribe("servers", this, this.callback);
  }

  render(props) {
    console.log("current players in this: " + this.server_list.current_players);
    console.log("current players in globalThis: " + globalThis.server_list.current_players);
    return <div {...this.props} id="not_chat" class="join_server">
      <div class="titlebar">
        <h3 class="title">Servers</h3>
        <p class="nowrap padding" style="font-size: 7pt;">There are currently { this.server_list.current_players } players online</p>
        <div class="spacer"></div>
        filter
        <div class="filter down"></div>
        <div class="refresh"></div>
      </div>
      <div class="filterbar">
        <p class="nowrap">Players: { this.server_list.minimum_players } - { this.server_list.maximum_players }</p>
        <div class="slider" minValue="0" maxValue="64" min={ this.server_list.minimum_players } max={ this.server_list.maximum_players }>
          <div class="range" style="left: 0%; width: *; margin-right: 6dip;"></div>
          <div class="handle start" style="left: -3dip;"></div>
          <div class="handle end" style="left: auto; right: 0dip; margin-right: 0dip;"></div>
        </div>
        <checkmark class="big checked" toggle /><p class="nowrap">Same version</p>
      </div>
      <div class="body mheight">
        <ServersTable />
      </div>
      <div class="titlebar">
        <h3 class="title"><output title_menu /></h3>
        <div class="spacer"></div>
        <div class="dropdown_menu closed">PLAY ONLINE</div>
      </div>
      <div class="body hflow">
        <div class="menu child-padding" style="visibility: hidden;">
          <div class="padding" overlay="ip.htm"><h4>JOIN IP</h4></div>
          <div class="padding"><h4>LAUNCH SERVER</h4></div>
          <div class="padding"><h4>PLAY ONLINE</h4></div>
          <div class="padding"><h4>PLAY SKIRMISH</h4></div>
        </div>
        <div class="expand" style="margin-right: 10px; ">
          <h3>MAP: <span id="map-name" style="color: #CE5135;"></span></h3>
          <div class="hflow" style=" height: *; vertical-align: bottom;">
            <div class="vflow expand child-padding">
              <p>Time Limit: <span id="time-limit"></span></p>
              <p>Vehicle Limit: <span id="vehicle-limit"></span></p>
              <p>Player Limit: <span id="player-limit"></span></p>
              <p>Mine Limit: <span id="mine-limit"></span></p>
              <p>Game Mode: <span id="game-mode"></span></p>
            </div>
            <div class="vflow expand child-padding">
              <p><checkmark class="checked" id="crates" />Crates</p>
              <p><checkmark class="checked" id="steam" />Steam Required</p>
              <p><checkmark class="checked" id="ranked" />Ranked</p>
              <p><checkmark class="checked" id="balance" />Auto Balance</p>
              <p><checkmark class="checked" id="infantry" />Infantry Only</p>
            </div>
          </div>
          <button class="green" style="bottom: 0px;" onclick="joinServer();">JOIN SERVER</button>
        </div>
        <video id="map_video" src="../../PreviewVids/Default.avi" loop />
      </div>
    </div>
  }

  ["on click at div.dropdown_menu"](evt, target) {
    target.classList.toggle('open');
    target.classList.toggle('closed');
    document.$('div.body > div.menu').style['visibility'] = target.classList.contains('closed') ? 'collapse' : 'visible';
  }

  ["on click at div.refresh"](evt, target) {
    globalThis.server_list.refresh_servers();
  }

  ["on click at div.filter"](evt, target) {
    var spoiler = this.$("div.filterbar");
    if (spoiler.style["visibility"] == "collapse") {
      spoiler.style["visibility"] = "visible";
    } else if (spoiler.style["visibility"] == "visible") {
      spoiler.style["visibility"] = "collapse";
    }
  }
}