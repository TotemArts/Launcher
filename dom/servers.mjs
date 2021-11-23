class ServersTable extends Element {

  currentItem = null;
  selectedItems;
  props;
  list;

  this(props) {
    let {list, ...rest} = props;
    super.this?.(rest);
    this.list = list;
    this.props = rest; 
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
    let list = [];
    let totalItems = this.totalItems();
    let {currentItem, selectedItems } = this;
    for( let index = 0; index <= totalItems; ++index ) {
      let item = this.itemAt(index);
      if(item) list.push(this.renderItem(item,item === currentItem, selectedItems?.has(item)));
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

  renderList(items)
  {
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
        { items }
      </tbody>
    </table>;
  }

  renderItem(item, isCurrent, isSelected) {
    return <tr key={item.key}>
              <th></th>
              <th>Server Name</th>
              <th>Map</th>
              <th>Players</th>
              <th>Ping</th>
            </tr>;
  }
  
  itemOfElement(element) {
    return this.itemAt(element.elementIndex);
  }

  onkeydown(evt) {
    switch(evt.code) {
      case "KeyDOWN" : 
        if(!this.currentItem) { 
          this.componentUpdate({ currentItem : this.itemOfElement(0) });
        } else {
          let index = this.indexOf(this.currentItem);
          if( ++index < this.totalItems() ) {
            this.componentUpdate({ currentItem : this.itemAt(index) });
            //this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyUP" : 
        if(!this.currentItem) { 
          this.componentUpdate({ currentItem : this.itemAt(this.list.length - 1) });
        } else {
          let index = this.indexOf(this.currentItem);
          if( --index >= 0 ) {
            this.componentUpdate({ currentItem : this.itemAt(index) });
            //this.vlist.navigate("advance",index);
          }
        }
        break;
      case "KeyEND" : 
        this.currentItem = this.itemAt(this.totalItems() - 1);
        //this.vlist.navigate("end");
        break;
      case "KeyHOME" : 
        this.currentItem = this.itemAt(0);
        //this.vlist.navigate("start");
        break;
      default:
        return false;
    }
    this.post(new Event("input", {bubbles:true}));
    return true;
  }

  setCurrentOption(child) {
    console.log("setCurrentOption");
    console.log(child);
    let option;
    for(let node = child; node; node = node.parentElement) {
      if(node.parentElement === this) {
        option = node;
        break; 
      }
    }
    if(option) {
      console.log(this.itemOfElement(option));
      this.componentUpdate({ currentItem : this.itemOfElement(option) });
      this.post(new Event("input", {bubbles:true}));
      return true;
    }
  }

  ["on mousedown"](evt) {
    console.log("mousedown");
    if(evt.button == 1) {
      console.log("mousedown click");
      this.setCurrentOption(evt.target);
    }
  }
  /*["on mousemove"](evt) {
    console.log("mousemove");
    if(evt.button == 1) {
      console.log("mousemove click");
      this.setCurrentOption(evt.target);
    }
  }*/

  get value() {
    if(!this.currentItem) return undefined;
    return this.currentItem;
  }
}

export class Servers extends Element 
{
  list = [
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}, 
    {key:"j",text:"hi"}];

  render(props) 
  {
    //let currentServer = props.current; // ChannelDriver
    //var list = Object.values(this.servers).map( (server) => <ServerCaption key={server.data.IP + ":" + server.data.Port} server={server} current={ server === currentServer } /> );
    //         {list}

    return <div {...this.props} id="not_chat" class="join_server">
    <div class="titlebar">
      <h3 class="title">Servers</h3>
      <p class="nowrap padding" style="font-size: 7pt;">There are currently <output players_online/> players online</p>
      <div class="spacer"></div>
      filter
      <div class="filter down"></div>
      <div class="refresh" onclick="Window.this.xcall('get_servers', getServersCallback);"></div>
    </div>
    <div class="filterbar">
      <p class="nowrap">Players: <output server_filter_players_min /> - <output server_filter_players_max /></p>
      <div class="slider" minValue="0" maxValue="64" min="0" max="64">
        <div class="range" style="left: 0%; width: *; margin-right: 6dip;"></div>
        <div class="handle start" style="left: -3dip;"></div>
        <div class="handle end" style="left: auto; right: 0dip; margin-right: 0dip;"></div>
      </div>
      <checkmark class="big checked" toggle/><p class="nowrap">Same version</p>
    </div>
    <div class="body mheight">
      <ServersTable list={this.list}></ServersTable>
    </div>
    <div class="titlebar">
      <h3 class="title"><output title_menu/></h3>
      <div class="spacer"></div>
      <div class="dropdown_menu closed" onclick="this.classList.toggle('open');this.classList.toggle('closed'); document.$('div.body > div.menu').style['visibility'] = this.classList.contains('closed')?'collapse':'visible';">PLAY ONLINE</div>
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
            <p><checkmark class="checked" id="crates"/>Crates</p>
            <p><checkmark class="checked" id="steam"/>Steam Required</p>
            <p><checkmark class="checked" id="ranked"/>Ranked</p>
            <p><checkmark class="checked" id="balance"/>Auto Balance</p>
            <p><checkmark class="checked" id="infantry"/>Infantry Only</p>
          </div>
        </div>
        <button class="green" style="bottom: 0px;" onclick="joinServer();">JOIN SERVER</button>
      </div>
      <video id="map_video" src="../../PreviewVids/Default.avi" loop/>
    </div>
  </div>
  }
}